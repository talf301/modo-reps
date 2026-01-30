---
phase: 01-capture-infrastructure-proof-of-concept
plan: 04B
type: execute
wave: 3
depends_on: ["01-04A"]
files_modified: [src/capture-status.tsx, src/App.tsx]
autonomous: true
user_setup: []

must_haves:
  truths:
    - "Capture status displays admin privilege and driver status"
    - "Start/Stop capture buttons render correctly"
    - "Status updates propagate from Rust backend"
  artifacts:
    - path: "src/capture-status.tsx"
      provides: "Capture status UI component"
      contains: "CaptureStatus"
    - path: "src/App.tsx"
      provides: "Main application component"
      contains: "CaptureStatus"
  key_links:
    - from: "src/capture-status.tsx"
      to: "src-tauri/src/ui/commands.rs"
      via: "Tauri invoke API"
      pattern: "invoke\\('(start|stop|get|check).*_"
---

<objective>
Create basic capture status UI with admin privilege display, WinDivert driver status, and start/stop capture buttons.

Purpose: Provide user interface for packet capture control, displaying admin status, driver status, and capture state with real-time updates.
Output: Capture status component with Tauri command integration for control and status queries.
</objective>

<execution_context>
@~/.config/opencode/get-shit-done/workflows/execute-plan.md
@~/.config/opencode/get-shit-done/templates/summary.md
</execution_context>

<context>
@.planning/PROJECT.md
@.planning/ROADMAP.md
@.planning/STATE.md
@.planning/REQUIREMENTS.md

# Prior Plan 01-01: Tauri project initialized with vanilla TypeScript template
# Prior Plan 01-02: Admin privilege and driver check Tauri command created (check_admin_privileges)
# Prior Plan 01-04A: Capture status types and Tauri commands defined (backend)
# This plan creates the capture status UI component (frontend only)
</context>

<tasks>

<task type="auto">
  <name>Task 1: Create capture status UI component</name>
  <files>src/capture-status.tsx</files>
  <action>
Create src/capture-status.tsx component:

```typescript
import { invoke } from '@tauri-apps/api/tauri';
import { useEffect, useState } from 'react';

interface AdminStatus {
  is_admin: boolean;
  can_capture: boolean;
  windivert_driver_found: boolean;
}

interface CaptureStatus {
  is_running: boolean;
  packet_count: number;
  bytes_per_second: number;
  last_packet_time: string | null;
}

export function CaptureStatus() {
  const [adminStatus, setAdminStatus] = useState<AdminStatus | null>(null);
  const [captureStatus, setCaptureStatus] = useState<CaptureStatus | null>(null);

  // Check admin privileges and driver on mount
  useEffect(() => {
    invoke<AdminStatus>('check_admin_privileges')
      .then(setAdminStatus)
      .catch(console.error);
  }, []);

  // Poll capture status every 500ms when running
  useEffect(() => {
    if (!captureStatus?.is_running) return;

    const interval = setInterval(async () => {
      try {
        const status = await invoke<CaptureStatus>('get_capture_status');
        setCaptureStatus(status);
      } catch (error) {
        console.error('Failed to get capture status:', error);
      }
    }, 500);

    return () => clearInterval(interval);
  }, [captureStatus?.is_running]);

  const handleStartCapture = async () => {
    try {
      const status = await invoke<CaptureStatus>('start_capture');
      setCaptureStatus(status);
    } catch (error) {
      alert(`Failed to start capture: ${error}`);
    }
  };

  const handleStopCapture = async () => {
    try {
      const status = await invoke<CaptureStatus>('stop_capture');
      setCaptureStatus(status);
    } catch (error) {
      alert(`Failed to stop capture: ${error}`);
    }
  };

  if (!adminStatus) {
    return <div>Loading status...</div>;
  }

  return (
    <div style={{ padding: '20px' }}>
      <h1>MTGO Replay Capture</h1>

      {/* Admin and Driver Status Section */}
      <div style={{ marginBottom: '20px', padding: '10px', border: '1px solid #ccc' }}>
        <h2>Privilege & Driver Status</h2>
        <p>
          Admin: <strong>{adminStatus.is_admin ? '✓ Yes' : '✗ No'}</strong>
          {adminStatus.is_admin && ' '}
          WinDivert Driver: <strong>{adminStatus.windivert_driver_found ? '✓ Found' : '✗ Not Found'}</strong>
          {!adminStatus.can_capture && (
            <div style={{ color: 'red', marginTop: '10px' }}>
              {!adminStatus.is_admin && <p>Please restart the application as Administrator to capture traffic.</p>}
              {!adminStatus.windivert_driver_found && <p>Please download WinDivert 2.2.2-A from https://reqrypt.org/windivert.html and place WinDivert.dll and WinDivert64.sys in the application directory.</p>}
            </div>
          )}
        </p>
      </div>

      {/* Capture Controls Section */}
      <div style={{ marginBottom: '20px', padding: '10px', border: '1px solid #ccc' }}>
        <h2>Capture Control</h2>
        <button
          onClick={handleStartCapture}
          disabled={!adminStatus.can_capture || captureStatus?.is_running}
          style={{
            padding: '8px 16px',
            marginRight: '10px',
            opacity: !adminStatus.can_capture ? 0.5 : 1,
          }}
        >
          Start Capture
        </button>
        <button
          onClick={handleStopCapture}
          disabled={!captureStatus?.is_running}
          style={{ padding: '8px 16px' }}
        >
          Stop Capture
        </button>
      </div>

      {/* Capture Status Section */}
      {captureStatus && (
        <div style={{ padding: '10px', border: '1px solid #ccc' }}>
          <h2>Capture Status</h2>
          <p>Status: <strong>{captureStatus.is_running ? 'Running' : 'Stopped'}</strong></p>
          <p>Packets Captured: <strong>{captureStatus.packet_count.toLocaleString()}</strong></p>
          <p>Throughput: <strong>{captureStatus.bytes_per_second.toFixed(2)} bytes/s</strong></p>
          <p>Last Packet: <strong>{captureStatus.last_packet_time || 'N/A'}</strong></p>
        </div>
      )}
    </div>
  );
}
```

DO NOT implement complex styling or animations in this phase - keep the UI simple and functional. The focus is on Tauri integration, not visual polish.
  </action>
  <verify>grep -E "(invoke|useEffect|useState|AdminStatus|windivert_driver_found)" src/capture-status.tsx</verify>
  <done>Capture status component created with admin and driver checks, start/stop buttons, and real-time status polling</done>
</task>

<task type="auto">
  <name>Task 2: Integrate component into main app</name>
  <files>src/App.tsx</files>
  <action>
Update src/App.tsx to use the CaptureStatus component:

```typescript
import { CaptureStatus } from './capture-status';

function App() {
  return <CaptureStatus />;
}

export default App;
```

If the default template has different file structure, ensure the CaptureStatus component is rendered as the main app content.
  </action>
  <verify>cargo tauri dev (app launches and shows capture status UI)</verify>
  <done>Capture status UI displays in Tauri application window</done>
</task>

</tasks>

<verification>
- Frontend component displays admin and driver status
- Start/Stop capture buttons work
- Status polling updates display every 500ms
- Application compiles and runs
- User sees clear error messages if admin privileges or WinDivert driver missing
</verification>

<success_criteria>
Capture status UI created with admin privilege and WinDivert driver display, start/stop capture buttons, and real-time status updates via Tauri commands.
</success_criteria>

<output>
After completion, create `.planning/phases/01-capture-infrastructure-proof-of-concept/01-04B-SUMMARY.md`
</output>
