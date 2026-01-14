import * as THREE from 'three';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';

// Setup
const canvas = document.getElementById('main-canvas') as HTMLCanvasElement;
const container = document.getElementById('main-view')!;

const renderer = new THREE.WebGLRenderer({ canvas, antialias: true, alpha: true });
renderer.setPixelRatio(window.devicePixelRatio);
renderer.setClearColor(0x000000, 1);

const scene = new THREE.Scene();

// Camera (Z-up coordinate system)
const camera = new THREE.PerspectiveCamera(45, 1, 0.1, 1000);
camera.up.set(0, 0, 1); // Z is up
camera.position.set(-80, -150, 80); // View from front-left, slightly above

// Controls
const controls = new OrbitControls(camera, canvas);
controls.enableDamping = true;
controls.dampingFactor = 0.05;
controls.target.set(0, 0, 20); // Look at slightly above origin

// Current mesh and field visualizations
let current_mesh: THREE.Mesh | null = null;
let arrows_group: THREE.Group | null = null;
let field_plane: THREE.Mesh | null = null;

// Graph canvas (defined in HTML, hidden by default)
const graph_canvas = document.getElementById('graph-canvas') as HTMLCanvasElement;
const graph_ctx = graph_canvas.getContext('2d')!;
const graph_window = document.getElementById('graph-window')!;

declare function openWindow(id: string): void;

// X-ray material (Fresnel-based transparency with vertex colors)
function create_xray_material(): THREE.ShaderMaterial {
  return new THREE.ShaderMaterial({
    uniforms: {
      fresnelPower: { value: 2.0 },
    },
    vertexShader: `
      attribute vec3 color;
      varying vec3 vNormal;
      varying vec3 vViewPosition;
      varying vec3 vColor;
      void main() {
        vNormal = normalize(normalMatrix * normal);
        vColor = color;
        vec4 mvPosition = modelViewMatrix * vec4(position, 1.0);
        vViewPosition = -mvPosition.xyz;
        gl_Position = projectionMatrix * mvPosition;
      }
    `,
    fragmentShader: `
      uniform float fresnelPower;
      varying vec3 vNormal;
      varying vec3 vViewPosition;
      varying vec3 vColor;
      void main() {
        vec3 viewDir = normalize(vViewPosition);
        float fresnel = pow(1.0 - abs(dot(viewDir, vNormal)), fresnelPower);
        // Minimum opacity of 0.15 when viewing straight on, max 0.85 at edges
        float opacity = 0.01 + fresnel * 0.5;
        gl_FragColor = vec4(vColor, opacity);
      }
    `,
    transparent: true,
    side: THREE.DoubleSide,
    depthWrite: false,
    blending: THREE.AdditiveBlending,
  });
}

// Subtle axes
const axes_group = new THREE.Group();
const axis_len = 50;
const axis_mat = new THREE.LineBasicMaterial({ color: 0x333333 });
[[1, 0, 0], [0, 1, 0], [0, 0, 1]].forEach(dir => {
  const geo = new THREE.BufferGeometry().setFromPoints([
    new THREE.Vector3(0, 0, 0),
    new THREE.Vector3(dir[0] * axis_len, dir[1] * axis_len, dir[2] * axis_len)
  ]);
  axes_group.add(new THREE.Line(geo, axis_mat));
});

// Z axis label (small italic scientific style)
const label_canvas = document.createElement('canvas');
label_canvas.width = 32;
label_canvas.height = 32;
const label_ctx = label_canvas.getContext('2d')!;
label_ctx.fillStyle = '#555555';
label_ctx.font = 'italic 24px "Times New Roman", serif';
label_ctx.textAlign = 'center';
label_ctx.textBaseline = 'middle';
label_ctx.fillText('z', 16, 16);

const label_texture = new THREE.CanvasTexture(label_canvas);
const label_material = new THREE.SpriteMaterial({ map: label_texture, transparent: true });
const z_label = new THREE.Sprite(label_material);
z_label.position.set(0, 0, axis_len + 3);
z_label.scale.set(4, 4, 1);
axes_group.add(z_label);

scene.add(axes_group);

// Color map function (viridis-like)
function value_to_color(t: number): THREE.Color {
  // Clamp t to [0, 1]
  t = Math.max(0, Math.min(1, t));

  // Simple viridis-like colormap
  const r = Math.max(0, Math.min(1, 0.267 + 0.329 * t + 2.0 * t * t - 1.6 * t * t * t));
  const g = Math.max(0, Math.min(1, 0.004 + 1.4 * t - 0.7 * t * t));
  const b = Math.max(0, Math.min(1, 0.329 + 0.7 * t - 1.0 * t * t + 0.5 * t * t * t));

  return new THREE.Color(r, g, b);
}

// Parse binary mesh
function parse_binary_mesh(buffer: ArrayBuffer): THREE.BufferGeometry | null {
  const view = new DataView(buffer);

  if (buffer.byteLength < 8) return null;

  const numVertices = view.getUint32(0, true);
  const numIndices = view.getUint32(4, true);

  if (numVertices === 0 || numIndices === 0) return null;

  const positionsOffset = 8;
  const normalsOffset = positionsOffset + numVertices * 3 * 4;
  const colorsOffset = normalsOffset + numVertices * 3 * 4;
  const indicesOffset = colorsOffset + numVertices * 3 * 4;

  const expectedSize = indicesOffset + numIndices * 4;
  if (buffer.byteLength < expectedSize) {
    console.error(`Buffer too small: ${buffer.byteLength} < ${expectedSize}`);
    return null;
  }

  const positions = new Float32Array(buffer, positionsOffset, numVertices * 3);
  const normals = new Float32Array(buffer, normalsOffset, numVertices * 3);
  const colors = new Float32Array(buffer, colorsOffset, numVertices * 3);
  const indices = new Uint32Array(buffer, indicesOffset, numIndices);

  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute('position', new THREE.BufferAttribute(positions.slice(), 3));
  geometry.setAttribute('normal', new THREE.BufferAttribute(normals.slice(), 3));
  geometry.setAttribute('color', new THREE.BufferAttribute(colors.slice(), 3));
  geometry.setIndex(new THREE.BufferAttribute(indices.slice(), 1));

  console.log(`Mesh: ${numVertices} vertices, ${numIndices / 3} triangles`);

  return geometry;
}

// Parse binary field data
function parse_field_data(buffer: ArrayBuffer) {
  const view = new DataView(buffer);
  let offset = 8; // Skip "FIELD\0\0\0" header

  // 2D slice dimensions
  const sliceWidth = view.getUint32(offset, true); offset += 4;
  const sliceHeight = view.getUint32(offset, true); offset += 4;

  // 2D slice bounds [x_min, x_max, z_min, z_max]
  const xMin = view.getFloat32(offset, true); offset += 4;
  const xMax = view.getFloat32(offset, true); offset += 4;
  const zMin = view.getFloat32(offset, true); offset += 4;
  const zMax = view.getFloat32(offset, true); offset += 4;

  const sliceSize = sliceWidth * sliceHeight;

  // 2D slice data
  const sliceBx = new Float32Array(buffer, offset, sliceSize);
  offset += sliceSize * 4;
  const sliceBz = new Float32Array(buffer, offset, sliceSize);
  offset += sliceSize * 4;
  const sliceMagnitude = new Float32Array(buffer, offset, sliceSize);
  offset += sliceSize * 4;

  // 3D arrows
  const numArrows = view.getUint32(offset, true); offset += 4;

  const arrowPositions = new Float32Array(buffer, offset, numArrows * 3);
  offset += numArrows * 3 * 4;
  const arrowVectors = new Float32Array(buffer, offset, numArrows * 3);
  offset += numArrows * 3 * 4;
  const arrowMagnitudes = new Float32Array(buffer, offset, numArrows);
  offset += numArrows * 4;

  // 1D line
  const linePoints = view.getUint32(offset, true); offset += 4;

  const lineZ = new Float32Array(buffer, offset, linePoints);
  offset += linePoints * 4;
  const lineBz = new Float32Array(buffer, offset, linePoints);

  return {
    slice: {
      width: sliceWidth,
      height: sliceHeight,
      bounds: [xMin, xMax, zMin, zMax],
      bx: sliceBx,
      bz: sliceBz,
      magnitude: sliceMagnitude,
    },
    arrows: {
      positions: arrowPositions,
      vectors: arrowVectors,
      magnitudes: arrowMagnitudes,
    },
    line: {
      z: lineZ,
      bz: lineBz,
    },
  };
}

// Create 3D arrow field visualization
function create_arrow_field(arrows: { positions: Float32Array, vectors: Float32Array, magnitudes: Float32Array }) {
  const group = new THREE.Group();

  const numArrows = arrows.positions.length / 3;
  const maxMag = Math.max(...Array.from(arrows.magnitudes));

  // Create instanced mesh for efficiency
  const arrowLength = 8;
  const arrowGeo = new THREE.ConeGeometry(1.5, arrowLength, 8);
  arrowGeo.translate(0, arrowLength / 2, 0);
  arrowGeo.rotateX(Math.PI / 2); // Point along +Z by default

  for (let i = 0; i < numArrows; i++) {
    const px = arrows.positions[i * 3];
    const py = arrows.positions[i * 3 + 1];
    const pz = arrows.positions[i * 3 + 2];

    const vx = arrows.vectors[i * 3];
    const vy = arrows.vectors[i * 3 + 1];
    const vz = arrows.vectors[i * 3 + 2];

    const mag = arrows.magnitudes[i];
    const t = mag / maxMag;

    const color = value_to_color(t);
    const material = new THREE.MeshBasicMaterial({ color, transparent: true, opacity: 0.8 });

    const arrow = new THREE.Mesh(arrowGeo, material);
    arrow.position.set(px, py, pz);

    // Orient arrow along field direction
    const dir = new THREE.Vector3(vx, vy, vz).normalize();
    const quaternion = new THREE.Quaternion();
    quaternion.setFromUnitVectors(new THREE.Vector3(0, 0, 1), dir);
    arrow.quaternion.copy(quaternion);

    // Scale by magnitude
    const scale = 0.3 + 0.7 * t;
    arrow.scale.setScalar(scale);

    group.add(arrow);
  }

  return group;
}

// Create 2D field plane visualization (XZ plane)
function create_field_plane(slice: { width: number, height: number, bounds: number[], magnitude: Float32Array }) {
  const { width, height, bounds, magnitude } = slice;
  const [xMin, xMax, zMin, zMax] = bounds;

  // Create texture from magnitude data
  const textureData = new Uint8Array(width * height * 4);
  const maxMag = Math.max(...Array.from(magnitude));

  for (let i = 0; i < width * height; i++) {
    const t = magnitude[i] / maxMag;
    const color = value_to_color(t);

    // Flip Y coordinate for texture
    const row = Math.floor(i / width);
    const col = i % width;
    const flippedIdx = (height - 1 - row) * width + col;

    textureData[flippedIdx * 4] = Math.floor(color.r * 255);
    textureData[flippedIdx * 4 + 1] = Math.floor(color.g * 255);
    textureData[flippedIdx * 4 + 2] = Math.floor(color.b * 255);
    textureData[flippedIdx * 4 + 3] = 200; // Semi-transparent
  }

  const texture = new THREE.DataTexture(textureData, width, height, THREE.RGBAFormat);
  texture.needsUpdate = true;

  const planeWidth = xMax - xMin;
  const planeHeight = zMax - zMin;

  const geometry = new THREE.PlaneGeometry(planeWidth, planeHeight);
  const material = new THREE.MeshBasicMaterial({
    map: texture,
    transparent: true,
    side: THREE.DoubleSide,
    depthWrite: false,
  });

  const plane = new THREE.Mesh(geometry, material);
  plane.position.set((xMin + xMax) / 2, 0, (zMin + zMax) / 2);
  plane.rotation.x = -Math.PI / 2; // Lay flat in XZ plane

  return plane;
}

// Draw 1D graph
function draw_graph(line: { z: Float32Array, bz: Float32Array }) {
  const ctx = graph_ctx;
  const width = graph_canvas.width;
  const height = graph_canvas.height;

  // Show the graph window
  openWindow('graph-window');

  // Clear
  ctx.fillStyle = 'rgba(0, 0, 0, 0.9)';
  ctx.fillRect(0, 0, width, height);

  // Find max |Bz|
  let max_bz = 0;
  let min_bz = 0;
  for (let i = 0; i < line.bz.length; i++) {
    max_bz = Math.max(max_bz, line.bz[i]);
    min_bz = Math.min(min_bz, line.bz[i]);
  }

  const z_min = line.z[0];
  const z_max = line.z[line.z.length - 1];

  // Draw axes
  ctx.strokeStyle = '#666';
  ctx.lineWidth = 1;
  ctx.beginPath();
  ctx.moveTo(40, 10);
  ctx.lineTo(40, height - 30);
  ctx.lineTo(width - 10, height - 30);
  ctx.stroke();

  // Draw zero line
  const zero_y = height - 30 - ((0 - min_bz) / (max_bz - min_bz)) * (height - 50);
  ctx.strokeStyle = '#444';
  ctx.setLineDash([3, 3]);
  ctx.beginPath();
  ctx.moveTo(40, zero_y);
  ctx.lineTo(width - 10, zero_y);
  ctx.stroke();
  ctx.setLineDash([]);

  // Draw data (white line for 80s TUI look)
  ctx.strokeStyle = '#ccc';
  ctx.lineWidth = 1;
  ctx.beginPath();

  for (let i = 0; i < line.z.length; i++) {
    const x = 40 + ((line.z[i] - z_min) / (z_max - z_min)) * (width - 60);
    const y = height - 30 - ((line.bz[i] - min_bz) / (max_bz - min_bz)) * (height - 50);

    if (i === 0) {
      ctx.moveTo(x, y);
    } else {
      ctx.lineTo(x, y);
    }
  }
  ctx.stroke();

  // Labels
  ctx.fillStyle = '#888';
  ctx.font = '10px "Courier New", monospace';
  ctx.fillText('Bz (mT)', 5, 15);
  ctx.fillText(`${(max_bz * 1000).toFixed(2)}`, 5, 28);
  ctx.fillText(`${(min_bz * 1000).toFixed(2)}`, 5, height - 35);
  ctx.fillText('Z (mm)', width - 50, height - 5);
  ctx.fillText(`${z_min.toFixed(0)}`, 35, height - 15);
  ctx.fillText(`${z_max.toFixed(0)}`, width - 30, height - 15);

  // Max field indicator
  ctx.fillStyle = '#ccc';
  ctx.font = '10px "Courier New", monospace';
  ctx.fillText(`MAX: ${(max_bz * 1000).toFixed(3)} mT`, width - 130, 15);

  // Center field
  const center_idx = Math.floor(line.z.length / 2);
  ctx.fillText(`CTR: ${(line.bz[center_idx] * 1000).toFixed(3)} mT`, width - 130, 28);
}

function update_mesh(buffer: ArrayBuffer) {
  // Check if this is field data
  const header = new Uint8Array(buffer, 0, 5);
  const headerStr = String.fromCharCode(...header);

  if (headerStr === 'FIELD') {
    // Parse and display field data
    const fieldData = parse_field_data(buffer);
    console.log(`Field data: ${fieldData.slice.width}x${fieldData.slice.height} slice, ${fieldData.arrows.positions.length / 3} arrows`);

    // Remove old visualizations
    if (arrows_group) {
      scene.remove(arrows_group);
      arrows_group = null;
    }
    if (field_plane) {
      scene.remove(field_plane);
      field_plane = null;
    }

    // Create new visualizations
    arrows_group = create_arrow_field(fieldData.arrows);
    scene.add(arrows_group);

    field_plane = create_field_plane(fieldData.slice);
    scene.add(field_plane);

    // Draw 1D graph
    draw_graph(fieldData.line);

    return;
  }

  // Regular mesh data - clear old mesh AND any field visualizations
  if (current_mesh) {
    current_mesh.geometry.dispose();
    if (current_mesh.material instanceof THREE.Material) {
      current_mesh.material.dispose();
    }
    scene.remove(current_mesh);
    current_mesh = null;
  }

  // Clear field visualizations from previous file
  if (arrows_group) {
    scene.remove(arrows_group);
    arrows_group = null;
  }
  if (field_plane) {
    scene.remove(field_plane);
    field_plane = null;
  }
  // Hide graph window
  graph_window.classList.remove('visible');

  const geometry = parse_binary_mesh(buffer);
  if (!geometry) return;

  const material = create_xray_material();
  current_mesh = new THREE.Mesh(geometry, material);
  scene.add(current_mesh);
}

// WebSocket
function connect_websocket() {
  const ws = new WebSocket(`ws://${window.location.hostname}:3001/ws`);
  ws.binaryType = 'arraybuffer';

  ws.onopen = () => console.log('WebSocket connected');

  ws.onmessage = (event) => {
    if (event.data instanceof ArrayBuffer) {
      update_mesh(event.data);
    }
  };

  ws.onclose = () => {
    console.log('Disconnected, reconnecting...');
    setTimeout(connect_websocket, 2000);
  };

  ws.onerror = (err) => console.error('WebSocket error:', err);
}

// Resize
function resize() {
  const w = container.clientWidth;
  const h = container.clientHeight;
  camera.aspect = w / h;
  camera.updateProjectionMatrix();
  renderer.setSize(w, h);
}
resize();
window.addEventListener('resize', resize);

// Render loop
function animate() {
  requestAnimationFrame(animate);
  controls.update();
  renderer.render(scene, camera);
}

// Start
connect_websocket();
animate();
