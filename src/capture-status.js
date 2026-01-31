const { invoke } = window.__TAURI__.core;

// State management
let adminStatus = null;
let captureStatus = null;
let captureStatusInterval = null;

// Initialize the application
document.addEventListener('DOMContentLoaded', () => {
  checkAdminPrivileges();
});

// Check admin privileges and driver status
async function checkAdminPrivileges() {
  try {
    adminStatus = await invoke('check_admin_privileges');
    updateUI();
  } catch (error) {
    console.error('Failed to check admin privileges:', error);
    document.getElementById('admin-status').textContent = 'Error checking privileges';
  }
}

// Start capture
async function startCapture() {
  if (!adminStatus?.can_capture) {
    alert('Cannot start capture: Admin privileges and WinDivert driver required');
    return;
  }

  try {
    captureStatus = await invoke('start_capture');
    updateUI();
    startStatusPolling();
  } catch (error) {
    alert(`Failed to start capture: ${error}`);
  }
}

// Stop capture
async function stopCapture() {
  try {
    captureStatus = await invoke('stop_capture');
    updateUI();
    stopStatusPolling();
  } catch (error) {
    alert(`Failed to stop capture: ${error}`);
  }
}

// Poll capture status every 500ms when running
function startStatusPolling() {
  if (captureStatusInterval) clearInterval(captureStatusInterval);

  captureStatusInterval = setInterval(async () => {
    try {
      const status = await invoke('get_capture_status');
      captureStatus = status;
      updateCaptureStatusDisplay();
    } catch (error) {
      console.error('Failed to get capture status:', error);
    }
  }, 500);
}

function stopStatusPolling() {
  if (captureStatusInterval) {
    clearInterval(captureStatusInterval);
    captureStatusInterval = null;
  }
}

// Update the UI based on current state
function updateUI() {
  updateAdminStatusDisplay();
  updateCaptureControls();
  updateCaptureStatusDisplay();
}

// Update admin and driver status display
function updateAdminStatusDisplay() {
  const adminStatusEl = document.getElementById('admin-status');

  if (!adminStatus) {
    adminStatusEl.textContent = 'Loading status...';
    return;
  }

  const adminIndicator = adminStatus.is_admin ? '✓ Yes' : '✗ No';
  const driverIndicator = adminStatus.windivert_driver_found ? '✓ Found' : '✗ Not Found';

  let html = `
    <h2>Privilege & Driver Status</h2>
    <p>
      Admin: <strong>${adminIndicator}</strong>
      ${adminStatus.is_admin ? ' ' : ''}
      WinDivert Driver: <strong>${driverIndicator}</strong>
    </p>
  `;

  if (!adminStatus.can_capture) {
    html += '<div style="color: red; margin-top: 10px;">';
    if (!adminStatus.is_admin) {
      html += '<p>Please restart the application as Administrator to capture traffic.</p>';
    }
    if (!adminStatus.windivert_driver_found) {
      html += '<p>Please download WinDivert 2.2.2-A from https://reqrypt.org/windivert.html and place WinDivert.dll and WinDivert64.sys in the application directory.</p>';
    }
    html += '</div>';
  }

  adminStatusEl.innerHTML = html;
}

// Update capture control buttons
function updateCaptureControls() {
  const startBtn = document.getElementById('start-capture-btn');
  const stopBtn = document.getElementById('stop-capture-btn');

  if (!adminStatus) return;

  startBtn.disabled = !adminStatus.can_capture || captureStatus?.is_running;
  startBtn.style.opacity = adminStatus.can_capture ? '1' : '0.5';

  stopBtn.disabled = !captureStatus?.is_running;
}

// Update capture status display
function updateCaptureStatusDisplay() {
  const captureStatusEl = document.getElementById('capture-status');

  if (!captureStatus) {
    captureStatusEl.innerHTML = '';
    return;
  }

  const statusIndicator = captureStatus.is_running ? 'Running' : 'Stopped';
  const packetCount = captureStatus.packet_count.toLocaleString();
  const throughput = captureStatus.bytes_per_second.toFixed(2);
  const lastPacket = captureStatus.last_packet_time || 'N/A';

  captureStatusEl.innerHTML = `
    <h2>Capture Status</h2>
    <p>Status: <strong>${statusIndicator}</strong></p>
    <p>Packets Captured: <strong>${packetCount}</strong></p>
    <p>Throughput: <strong>${throughput} bytes/s</strong></p>
    <p>Last Packet: <strong>${lastPacket}</strong></p>
  `;
}

// Set up event listeners for buttons after DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
  const startBtn = document.getElementById('start-capture-btn');
  const stopBtn = document.getElementById('stop-capture-btn');

  if (startBtn) {
    startBtn.addEventListener('click', startCapture);
  }
  if (stopBtn) {
    stopBtn.addEventListener('click', stopCapture);
  }
});
