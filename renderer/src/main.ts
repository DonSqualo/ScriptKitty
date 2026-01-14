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
let currentMesh: THREE.Mesh | null = null;
let arrowsGroup: THREE.Group | null = null;
let fieldPlane: THREE.Mesh | null = null;

// Create graph canvas for 1D plot
const graphCanvas = document.createElement('canvas');
graphCanvas.id = 'graph-canvas';
graphCanvas.style.cssText = `
  position: absolute;
  bottom: 20px;
  right: 20px;
  width: 300px;
  height: 200px;
  background: rgba(0, 0, 0, 0.8);
  border: 1px solid #333;
  border-radius: 4px;
`;
document.body.appendChild(graphCanvas);
const graphCtx = graphCanvas.getContext('2d')!;
graphCanvas.width = 300;
graphCanvas.height = 200;

// X-ray material (Fresnel-based transparency)
function createXrayMaterial(): THREE.ShaderMaterial {
  return new THREE.ShaderMaterial({
    uniforms: {
      color: { value: new THREE.Color(0xffffff) },
      fresnelPower: { value: 2.0 },
    },
    vertexShader: `
      varying vec3 vNormal;
      varying vec3 vViewPosition;
      void main() {
        vNormal = normalize(normalMatrix * normal);
        vec4 mvPosition = modelViewMatrix * vec4(position, 1.0);
        vViewPosition = -mvPosition.xyz;
        gl_Position = projectionMatrix * mvPosition;
      }
    `,
    fragmentShader: `
      uniform vec3 color;
      uniform float fresnelPower;
      varying vec3 vNormal;
      varying vec3 vViewPosition;
      void main() {
        vec3 viewDir = normalize(vViewPosition);
        float fresnel = pow(1.0 - abs(dot(viewDir, vNormal)), fresnelPower);
        // Minimum opacity of 0.15 when viewing straight on, max 0.85 at edges
        float opacity = 0.01 + fresnel * 0.5;
        gl_FragColor = vec4(color, opacity);
      }
    `,
    transparent: true,
    side: THREE.DoubleSide,
    depthWrite: false,
    blending: THREE.AdditiveBlending,
  });
}

// Subtle axes
const axesGroup = new THREE.Group();
const axisLen = 50;
const axisMat = new THREE.LineBasicMaterial({ color: 0x333333 });
[[1, 0, 0], [0, 1, 0], [0, 0, 1]].forEach(dir => {
  const geo = new THREE.BufferGeometry().setFromPoints([
    new THREE.Vector3(0, 0, 0),
    new THREE.Vector3(dir[0] * axisLen, dir[1] * axisLen, dir[2] * axisLen)
  ]);
  axesGroup.add(new THREE.Line(geo, axisMat));
});

// Z axis label
const labelCanvas = document.createElement('canvas');
labelCanvas.width = 64;
labelCanvas.height = 64;
const labelCtx = labelCanvas.getContext('2d')!;
labelCtx.fillStyle = '#666666';
labelCtx.font = 'bold 48px monospace';
labelCtx.textAlign = 'center';
labelCtx.textBaseline = 'middle';
labelCtx.fillText('Z', 32, 32);

const labelTexture = new THREE.CanvasTexture(labelCanvas);
const labelMaterial = new THREE.SpriteMaterial({ map: labelTexture, transparent: true });
const zLabel = new THREE.Sprite(labelMaterial);
zLabel.position.set(0, 0, axisLen + 5);
zLabel.scale.set(8, 8, 1);
axesGroup.add(zLabel);

scene.add(axesGroup);

// Color map function (viridis-like)
function valueToColor(t: number): THREE.Color {
  // Clamp t to [0, 1]
  t = Math.max(0, Math.min(1, t));

  // Simple viridis-like colormap
  const r = Math.max(0, Math.min(1, 0.267 + 0.329 * t + 2.0 * t * t - 1.6 * t * t * t));
  const g = Math.max(0, Math.min(1, 0.004 + 1.4 * t - 0.7 * t * t));
  const b = Math.max(0, Math.min(1, 0.329 + 0.7 * t - 1.0 * t * t + 0.5 * t * t * t));

  return new THREE.Color(r, g, b);
}

// Parse binary mesh
function parseBinaryMesh(buffer: ArrayBuffer): THREE.BufferGeometry | null {
  const view = new DataView(buffer);

  if (buffer.byteLength < 8) return null;

  const numVertices = view.getUint32(0, true);
  const numIndices = view.getUint32(4, true);

  if (numVertices === 0 || numIndices === 0) return null;

  const positionsOffset = 8;
  const normalsOffset = positionsOffset + numVertices * 3 * 4;
  const indicesOffset = normalsOffset + numVertices * 3 * 4;

  const expectedSize = indicesOffset + numIndices * 4;
  if (buffer.byteLength < expectedSize) {
    console.error(`Buffer too small: ${buffer.byteLength} < ${expectedSize}`);
    return null;
  }

  const positions = new Float32Array(buffer, positionsOffset, numVertices * 3);
  const normals = new Float32Array(buffer, normalsOffset, numVertices * 3);
  const indices = new Uint32Array(buffer, indicesOffset, numIndices);

  const geometry = new THREE.BufferGeometry();
  geometry.setAttribute('position', new THREE.BufferAttribute(positions.slice(), 3));
  geometry.setAttribute('normal', new THREE.BufferAttribute(normals.slice(), 3));
  geometry.setIndex(new THREE.BufferAttribute(indices.slice(), 1));

  console.log(`Mesh: ${numVertices} vertices, ${numIndices / 3} triangles`);

  return geometry;
}

// Parse binary field data
function parseFieldData(buffer: ArrayBuffer) {
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
function createArrowField(arrows: { positions: Float32Array, vectors: Float32Array, magnitudes: Float32Array }) {
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

    const color = valueToColor(t);
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
function createFieldPlane(slice: { width: number, height: number, bounds: number[], magnitude: Float32Array }) {
  const { width, height, bounds, magnitude } = slice;
  const [xMin, xMax, zMin, zMax] = bounds;

  // Create texture from magnitude data
  const textureData = new Uint8Array(width * height * 4);
  const maxMag = Math.max(...Array.from(magnitude));

  for (let i = 0; i < width * height; i++) {
    const t = magnitude[i] / maxMag;
    const color = valueToColor(t);

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
function drawGraph(line: { z: Float32Array, bz: Float32Array }) {
  const ctx = graphCtx;
  const width = graphCanvas.width;
  const height = graphCanvas.height;

  // Clear
  ctx.fillStyle = 'rgba(0, 0, 0, 0.9)';
  ctx.fillRect(0, 0, width, height);

  // Find max |Bz|
  let maxBz = 0;
  let minBz = 0;
  for (let i = 0; i < line.bz.length; i++) {
    maxBz = Math.max(maxBz, line.bz[i]);
    minBz = Math.min(minBz, line.bz[i]);
  }

  const maxVal = Math.max(Math.abs(maxBz), Math.abs(minBz));
  const zMin = line.z[0];
  const zMax = line.z[line.z.length - 1];

  // Draw axes
  ctx.strokeStyle = '#444';
  ctx.lineWidth = 1;
  ctx.beginPath();
  ctx.moveTo(40, 10);
  ctx.lineTo(40, height - 30);
  ctx.lineTo(width - 10, height - 30);
  ctx.stroke();

  // Draw zero line
  const zeroY = height - 30 - ((0 - minBz) / (maxBz - minBz)) * (height - 50);
  ctx.strokeStyle = '#333';
  ctx.setLineDash([5, 5]);
  ctx.beginPath();
  ctx.moveTo(40, zeroY);
  ctx.lineTo(width - 10, zeroY);
  ctx.stroke();
  ctx.setLineDash([]);

  // Draw data
  ctx.strokeStyle = '#00ff88';
  ctx.lineWidth = 2;
  ctx.beginPath();

  for (let i = 0; i < line.z.length; i++) {
    const x = 40 + ((line.z[i] - zMin) / (zMax - zMin)) * (width - 60);
    const y = height - 30 - ((line.bz[i] - minBz) / (maxBz - minBz)) * (height - 50);

    if (i === 0) {
      ctx.moveTo(x, y);
    } else {
      ctx.lineTo(x, y);
    }
  }
  ctx.stroke();

  // Labels
  ctx.fillStyle = '#888';
  ctx.font = '11px monospace';
  ctx.fillText('Bz (mT)', 5, 20);
  ctx.fillText(`${(maxBz * 1000).toFixed(2)}`, 5, 25);
  ctx.fillText(`${(minBz * 1000).toFixed(2)}`, 5, height - 35);
  ctx.fillText('Z (mm)', width - 50, height - 5);
  ctx.fillText(`${zMin.toFixed(0)}`, 35, height - 15);
  ctx.fillText(`${zMax.toFixed(0)}`, width - 30, height - 15);

  // Max field indicator
  ctx.fillStyle = '#00ff88';
  ctx.font = 'bold 14px monospace';
  ctx.fillText(`Max: ${(maxBz * 1000).toFixed(3)} mT`, width - 130, 25);

  // Center field
  const centerIdx = Math.floor(line.z.length / 2);
  ctx.fillText(`Center: ${(line.bz[centerIdx] * 1000).toFixed(3)} mT`, width - 150, 45);
}

function updateMesh(buffer: ArrayBuffer) {
  // Check if this is field data
  const header = new Uint8Array(buffer, 0, 5);
  const headerStr = String.fromCharCode(...header);

  if (headerStr === 'FIELD') {
    // Parse and display field data
    const fieldData = parseFieldData(buffer);
    console.log(`Field data: ${fieldData.slice.width}x${fieldData.slice.height} slice, ${fieldData.arrows.positions.length / 3} arrows`);

    // Remove old visualizations
    if (arrowsGroup) {
      scene.remove(arrowsGroup);
      arrowsGroup = null;
    }
    if (fieldPlane) {
      scene.remove(fieldPlane);
      fieldPlane = null;
    }

    // Create new visualizations
    arrowsGroup = createArrowField(fieldData.arrows);
    scene.add(arrowsGroup);

    fieldPlane = createFieldPlane(fieldData.slice);
    scene.add(fieldPlane);

    // Draw 1D graph
    drawGraph(fieldData.line);

    return;
  }

  // Regular mesh data
  if (currentMesh) {
    currentMesh.geometry.dispose();
    if (currentMesh.material instanceof THREE.Material) {
      currentMesh.material.dispose();
    }
    scene.remove(currentMesh);
    currentMesh = null;
  }

  const geometry = parseBinaryMesh(buffer);
  if (!geometry) return;

  const material = createXrayMaterial();
  currentMesh = new THREE.Mesh(geometry, material);
  scene.add(currentMesh);
}

// WebSocket
function connectWebSocket() {
  const ws = new WebSocket(`ws://${window.location.hostname}:3001/ws`);
  ws.binaryType = 'arraybuffer';

  ws.onopen = () => console.log('WebSocket connected');

  ws.onmessage = (event) => {
    if (event.data instanceof ArrayBuffer) {
      updateMesh(event.data);
    }
  };

  ws.onclose = () => {
    console.log('Disconnected, reconnecting...');
    setTimeout(connectWebSocket, 2000);
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
connectWebSocket();
animate();
