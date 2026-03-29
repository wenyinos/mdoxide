# PRD: Ket — Native Quantum Circuit Editor & Simulator

## Overview

**Ket** is a native desktop quantum circuit visualizer, editor, and simulator built in Rust with egui. It fills a gap in the quantum computing tooling landscape: no open-source, cross-platform, native desktop application exists for visually building quantum circuits and simulating them locally.

Every existing quantum circuit tool is either a Python matplotlib plot (static), a web app (Qiskit Composer, Quirk), or a terminal UI (OpenAPI-TUI). Ket is the first native GUI approach, offering instant startup, offline operation, and real-time visual feedback at 60fps.

**Working name**: Ket (from Dirac bra-ket notation ⟨ψ| and |ψ⟩, the standard quantum mechanics notation)

### Target Users

1. **Students** learning quantum computing — visual feedback makes abstract concepts tangible
2. **Researchers** prototyping algorithms — faster iteration than writing Python scripts
3. **Educators** teaching quantum concepts — live demos, step-through mode
4. **Developers** exploring quantum programming — bridge from classical to quantum thinking

### Inspiration & Differentiation

| Tool | Visual Editor? | Native Desktop? | Open Source? | Real-time Sim? |
|------|---------------|-----------------|-------------|----------------|
| **Ket** | Yes (egui) | Yes (Rust) | Yes | Yes |
| Qiskit Composer | Yes | No (web) | Partially | Cloud-based |
| Quirk | Yes | No (web) | Yes | Yes (JS) |
| QPanda | No (code-only) | No | Yes | Yes |
| QVNT | No (CLI) | Terminal | Yes | Yes |

**Key differentiator**: Native performance + visual editor + local simulation in a single app. No browser, no Python environment, no cloud dependency.

---

## Tech Stack

| Component | Technology | Rationale |
|-----------|-----------|-----------|
| Language | Rust | Performance, safety, cross-platform, team expertise |
| GUI | egui + eframe | Immediate-mode, fast iteration, proven in Ferrite |
| Quantum Sim | QuantRS2 (or custom) | Rust-native, state vector + stabilizer sims, MIT/Apache |
| Math | nalgebra or ndarray | Complex matrix operations for gate math |
| Serialization | serde + serde_json | Circuit save/load, export formats |
| Persistence | dirs + serde | Cross-platform config/session storage |

### Why egui?

- Proven: Ferrite is a 30k+ LOC egui application with custom rendering, graph layout, and interactive widgets
- Transferable: Graph layout (Sugiyama), node rendering, edge routing code from Ferrite's mermaid engine can inform circuit layout
- Fast: Immediate-mode means real-time simulation feedback without framework overhead
- Cross-platform: Windows, macOS, Linux from single codebase

### QuantRS2 as Simulation Backend

QuantRS2 (v0.1.2, Jan 2026) provides:
- `Circuit<N>` — compile-time qubit count, gate application
- `StateVectorSimulator` — full state vector simulation
- Stabilizer simulator — efficient for Clifford circuits
- Memory estimation utilities (`max_qubits_for_memory`)
- Feature-gated modules (circuit, sim, anneal, ml)
- MIT/Apache licensed

If QuantRS2 proves insufficient, a minimal custom state vector simulator is ~300 lines of Rust (complex vector + gate matrices). The visual editor is the hard part, not the simulation.

---

## Phase 1: Foundation — Circuit Model & Basic Rendering (MVP)

**Goal**: Render quantum circuits visually and simulate them. No editing yet — circuits defined in code or loaded from file.

### 1.1 Data Model

```rust
struct Circuit {
    qubits: Vec<QubitWire>,       // Named qubit wires
    gates: Vec<PlacedGate>,       // Gates positioned on the circuit
    measurements: Vec<Measurement>,
}

struct QubitWire {
    id: usize,
    label: String,                // e.g., "|q₀⟩"
}

struct PlacedGate {
    gate: GateType,
    target_qubits: Vec<usize>,   // Which wires this gate spans
    control_qubits: Vec<usize>,  // Control dots (for CNOT, Toffoli, etc.)
    column: usize,               // Time step position (left-to-right)
    parameters: Vec<f64>,        // For parameterized gates (Rx, Ry, Rz)
}

enum GateType {
    // Single-qubit gates
    H,                           // Hadamard — box with "H"
    X,                           // Pauli-X (NOT) — ⊕ circle symbol
    Y,                           // Pauli-Y — box with "Y"
    Z,                           // Pauli-Z — box with "Z"
    S, T,                        // Phase gates
    Rx(f64), Ry(f64), Rz(f64),  // Rotation gates
    // Multi-qubit gates
    CNOT,                        // Controlled-NOT — dot + ⊕
    CZ,                          // Controlled-Z — dot + dot
    SWAP,                        // Swap — ✕ + ✕ connected
    Toffoli,                     // CC-NOT — two dots + ⊕
    // Meta
    Barrier,                     // Visual separator
    Identity,                    // No-op wire
    Custom(String),              // User-defined gate
}
```

### 1.2 Circuit Rendering

Standard quantum circuit notation:
- **Horizontal lines** = qubit wires, flowing left-to-right (time axis)
- **Boxes** = single-qubit gates (labeled H, Y, Z, S, T, Rx, etc.)
- **⊕ symbol** = NOT/X gate target
- **● (filled dot)** = control qubit
- **○ (hollow dot)** = anti-control (gate fires when qubit is |0⟩)
- **Lines between qubits** = multi-qubit gate connections
- **Meter symbol** = measurement
- **Double line** = classical bit output after measurement

Layout rules:
- Gates at the same column (time step) execute simultaneously
- Wires labeled on the left: |q₀⟩, |q₁⟩, etc.
- Output state/probabilities shown on the right
- Grid-based layout with configurable spacing

### 1.3 Basic Simulation

- Integrate QuantRS2 `StateVectorSimulator` (or build minimal custom one)
- Run simulation on circuit change
- Display output state vector as probability bars next to each wire
- Show overall state as a probability histogram below the circuit
- Memory estimation shown in status bar ("12 qubits — ~64KB state vector")
- Warn when approaching RAM limits

### 1.4 File Format

JSON-based circuit format:

```json
{
  "ket_version": "0.1.0",
  "name": "Bell State",
  "description": "Creates an entangled Bell state |Φ+⟩",
  "qubits": 2,
  "gates": [
    { "type": "H", "targets": [0], "column": 0 },
    { "type": "CNOT", "controls": [0], "targets": [1], "column": 1 }
  ],
  "measurements": [
    { "qubit": 0, "column": 2 },
    { "qubit": 1, "column": 2 }
  ]
}
```

### Phase 1 Deliverables

- [ ] Project scaffolding (Cargo workspace, eframe app shell, CI)
- [ ] Circuit data model (`Circuit`, `PlacedGate`, `GateType`)
- [ ] Circuit renderer (qubit wires, gate symbols, connections, measurements)
- [ ] QuantRS2 integration for state vector simulation
- [ ] Probability histogram output display
- [ ] JSON circuit load/save
- [ ] 5 example circuits (Bell state, GHZ, Deutsch-Jozsa, teleportation, Grover 2-qubit)
- [ ] Status bar (qubit count, gate count, memory estimate)

---

## Phase 2: Interactive Visual Editor

**Goal**: Drag-and-drop circuit construction. This is the core differentiator.

### 2.1 Gate Palette

Side panel with categorized gates:

```
┌─ Gate Palette ─────────┐
│ ▸ Basic                │
│   [H] [X] [Y] [Z]     │
│ ▸ Phase                │
│   [S] [T] [S†] [T†]   │
│ ▸ Rotation             │
│   [Rx] [Ry] [Rz]      │
│ ▸ Multi-Qubit          │
│   [CNOT] [CZ] [SWAP]  │
│   [Toffoli] [Fredkin]  │
│ ▸ Measurement          │
│   [Measure] [Barrier]  │
│ ▸ Custom               │
│   [+ New Gate...]      │
└────────────────────────┘
```

### 2.2 Drag-and-Drop

- Drag gate from palette onto a qubit wire at a specific time step
- Visual drop indicators (highlight valid positions)
- Multi-qubit gates: drag to first qubit, then click second qubit to connect
- Right-click gate to edit parameters, delete, or copy
- Drag existing gates to reorder
- Ctrl+Click to select multiple gates
- Delete key removes selected gates

### 2.3 Wire Management

- Add/remove qubit wires (+ button below last wire, X button on wire label)
- Rename wires (click label to edit)
- Reorder wires by dragging labels
- Auto-compact: remove empty columns when gates are deleted

### 2.4 Undo/Redo

- Operation-based undo (same pattern as Ferrite's EditHistory)
- Operations: `AddGate`, `RemoveGate`, `MoveGate`, `AddWire`, `RemoveWire`, `EditParameter`
- Ctrl+Z / Ctrl+Y keybindings
- Undo stack displayed in sidebar (optional)

### 2.5 Real-Time Simulation Feedback

- Simulation runs automatically on every circuit edit (debounced ~100ms)
- State vector updates live as gates are added/removed
- For >20 qubits: simulation becomes manual (play button) due to compute time
- Progress indicator for long simulations
- Simulation runs on background thread, doesn't block UI

### Phase 2 Deliverables

- [ ] Gate palette panel with categorized gates
- [ ] Drag-and-drop gate placement
- [ ] Multi-qubit gate connection workflow
- [ ] Gate context menu (edit parameters, delete, copy)
- [ ] Wire add/remove/rename/reorder
- [ ] Undo/redo system
- [ ] Real-time simulation with debounced updates
- [ ] Background simulation thread
- [ ] Keyboard shortcuts (Delete, Ctrl+Z, Ctrl+Y, Ctrl+C, Ctrl+V)

---

## Phase 3: Visualization & Analysis

**Goal**: Rich visualization of quantum states to make the abstract tangible.

### 3.1 State Vector Display

- **Probability bars**: Horizontal bars next to each computational basis state, colored by phase
- **Phase wheel**: Color encoding (red = 0°, blue = 180°, etc.) following standard convention
- **Amplitude table**: Expandable table showing complex amplitudes for each basis state
- Toggle between: Probabilities only | Amplitudes | Both

### 3.2 Bloch Sphere

- 3D Bloch sphere visualization for single-qubit states
- Shows the qubit's state as a point on the sphere surface
- |0⟩ = north pole, |1⟩ = south pole, |+⟩ = equator
- Rendered with egui's painter (projected 3D → 2D, similar to Ferrite's mermaid rendering)
- Interactive rotation (drag to orbit)
- One Bloch sphere per qubit (reduced density matrix for multi-qubit states)

### 3.3 Step-Through Mode

- "Step" button advances one gate at a time
- Current gate highlighted with a vertical cursor line
- State vector updates at each step
- Allows tracing exactly how each gate transforms the state
- Playback controls: Step forward, Step back, Play (auto-advance), Reset
- Adjustable playback speed

### 3.4 Entanglement Visualization

- Color-coded qubit wires showing entanglement groups
- After a CNOT, entangled qubits share a color
- Concurrence or entanglement entropy metric displayed
- Helps students understand when and how entanglement forms

### 3.5 Circuit Statistics Panel

- Gate count (total, by type)
- Circuit depth (longest path)
- Qubit count
- Estimated execution time on real hardware (rough approximation)
- Memory usage for simulation
- Entanglement graph summary

### Phase 3 Deliverables

- [ ] Probability bar visualization with phase coloring
- [ ] Amplitude table view
- [ ] Bloch sphere rendering (2D projection)
- [ ] Step-through mode with playback controls
- [ ] Entanglement visualization (color-coded wires)
- [ ] Circuit statistics panel
- [ ] Toggle between visualization modes

---

## Phase 4: Export, Import & Ecosystem

**Goal**: Interoperate with the existing quantum ecosystem.

### 4.1 Export Formats

| Format | Target | Use Case |
|--------|--------|----------|
| Qiskit Python | `from qiskit import QuantumCircuit` | Run on IBM hardware |
| QASM 2.0/3.0 | OpenQASM standard | Universal interchange |
| QPanda Python | `from pyqpanda import *` | Run on Origin Quantum hardware |
| Cirq Python | `import cirq` | Run on Google hardware |
| LaTeX (quantikz) | `\begin{quantikz}` | Publication-quality diagrams |
| SVG | Vector graphic | Documentation, slides |
| PNG | Raster image | Quick sharing |

### 4.2 Import Formats

- Ket JSON (native format)
- OpenQASM 2.0 (most universal quantum circuit format)
- Qiskit Python (parse simple circuit construction code)
- JSON files from Quirk (popular web-based editor)

### 4.3 Example Library

Built-in circuit library organized by category:

- **Fundamentals**: Bell state, GHZ state, superposition, teleportation
- **Algorithms**: Deutsch-Jozsa, Bernstein-Vazirani, Grover (2-4 qubits), Quantum Fourier Transform
- **Error Correction**: Bit-flip code, Phase-flip code, Shor code
- **Applications**: Simple VQE circuit, QAOA template, quantum random walk

Each example includes title, description, and annotated explanation.

### Phase 4 Deliverables

- [ ] Qiskit Python export
- [ ] OpenQASM 2.0/3.0 export
- [ ] QPanda Python export
- [ ] LaTeX quantikz export
- [ ] SVG/PNG image export
- [ ] OpenQASM import
- [ ] Built-in example library (15+ circuits)
- [ ] Example browser with search and categories

---

## Phase 5: Advanced Features (Post-1.0)

### 5.1 Noise Simulation

- Depolarizing, amplitude damping, phase damping noise models
- Per-gate error rates configurable
- Density matrix simulation mode (vs pure state vector)
- Visual noise indicators on gates
- Comparison view: ideal vs noisy output

### 5.2 Parameterized Circuits

- Slider-controlled parameters on rotation gates (Rx(θ), Ry(θ), Rz(θ))
- Real-time simulation updates as sliders move
- Parameter sweep: plot output probability vs parameter value
- Variational circuit templates

### 5.3 Custom Gate Definitions

- Define custom gates as sub-circuits
- Gate decomposition view (expand custom gate into primitives)
- Save custom gates to personal library
- Share gate definitions as JSON

### 5.4 Tutorial Mode

- Guided interactive lessons built into the app
- Step-by-step walkthroughs of fundamental concepts
- Quiz/challenge mode: "Build a circuit that produces |Φ+⟩"
- Progress tracking

### 5.5 Collaboration

- Share circuits via URL (hosted service, optional)
- Embed circuit viewer in web pages (WASM export of renderer)

---

## Architecture

```
ket/
├── Cargo.toml              # Workspace root
├── crates/
│   ├── ket-core/           # Circuit data model, gate definitions
│   │   ├── circuit.rs      # Circuit, PlacedGate, GateType
│   │   ├── gates.rs        # Gate matrices, properties
│   │   ├── wire.rs         # QubitWire management
│   │   └── format/         # Serialization (JSON, QASM, Qiskit)
│   ├── ket-sim/            # Simulation engine
│   │   ├── statevector.rs  # State vector simulator
│   │   ├── stabilizer.rs   # Clifford circuit simulator
│   │   ├── noise.rs        # Noise models (Phase 5)
│   │   └── analysis.rs     # Entanglement, statistics
│   ├── ket-gui/            # egui application
│   │   ├── app.rs          # Main app, layout, event loop
│   │   ├── circuit_view.rs # Circuit rendering widget
│   │   ├── gate_palette.rs # Gate palette panel
│   │   ├── state_view.rs   # State vector / probability visualization
│   │   ├── bloch.rs        # Bloch sphere rendering
│   │   ├── editor.rs       # Drag-and-drop interaction logic
│   │   ├── examples.rs     # Built-in circuit browser
│   │   ├── export.rs       # Export dialogs
│   │   ├── theme.rs        # Light/dark themes
│   │   └── history.rs      # Undo/redo
│   └── ket-cli/            # Optional CLI for headless simulation
│       └── main.rs         # Run circuits from command line
├── examples/               # .ket.json circuit files
├── assets/                 # Icons, fonts
└── docs/                   # User documentation
```

### Why a Cargo Workspace?

- **ket-core** can be used as a library by other Rust projects
- **ket-sim** can be swapped or benchmarked independently
- **ket-gui** depends on the other crates but doesn't pollute them with egui
- **ket-cli** enables CI testing and scripting without a GUI

---

## UX Design Principles

1. **Circuit-first**: The circuit diagram is the hero — large, central, always visible
2. **Immediate feedback**: Every edit triggers simulation; results visible within 100ms
3. **Progressive complexity**: Start with H and CNOT; advanced features discoverable but not overwhelming
4. **Standard notation**: Follow established quantum circuit diagram conventions (Nielsen & Chuang)
5. **Keyboard-friendly**: Power users can build circuits without touching the mouse
6. **Educational affordances**: Tooltips explain what each gate does mathematically; step-through mode shows state evolution

### Default Window Layout

```
┌──────────────────────────────────────────────────────────┐
│  Ket — Quantum Circuit Editor              [_][□][✕]     │
├──────────┬───────────────────────────┬───────────────────┤
│          │                           │                   │
│  Gate    │   Circuit Diagram         │  State Vector     │
│  Palette │                           │  Visualization    │
│          │   |q₀⟩ ─[H]─●────[M]─    │                   │
│  [H]     │            │             │  |00⟩  ▓▓▓▓ 50%   │
│  [X]     │   |q₁⟩ ───⊕────[M]─    │  |01⟩       0%    │
│  [Y]     │                           │  |10⟩       0%    │
│  [Z]     │                           │  |11⟩  ▓▓▓▓ 50%  │
│  ...     │                           │                   │
│          │                           │  [Bloch] [Table]  │
├──────────┴───────────────────────────┴───────────────────┤
│  ► Step  ⏸ Pause  ⏮ Reset  │ 2 qubits │ 3 gates │ 64B  │
└──────────────────────────────────────────────────────────┘
```

---

## Performance Requirements

| Metric | Target |
|--------|--------|
| Startup time | < 500ms |
| Gate placement feedback | < 16ms (60fps) |
| Simulation (≤15 qubits) | < 100ms (real-time) |
| Simulation (≤25 qubits) | < 5s (background thread) |
| Simulation (≤30 qubits) | < 60s (with progress bar) |
| Memory (idle, no circuit) | < 30MB |
| Memory (20-qubit circuit) | < 50MB (16MB state vector + overhead) |
| Binary size | < 20MB |

---

## Open Questions

1. **QuantRS2 vs custom simulator?** QuantRS2 is young (v0.1.2). May need a custom state vector sim for reliability. Evaluate during Phase 1.
2. **WASM target?** egui supports WASM. A web version would dramatically increase reach. Consider as Phase 5+ goal.
3. **Project name**: "Ket" is clean but may conflict with existing projects. Alternatives: Eigenstate, Qubit Studio, CircuitQ, Superpose.
4. **License**: MIT? Apache-2.0? Dual MIT/Apache like most Rust ecosystem?
5. **Separate repo**: This is a standalone project, not part of Ferrite. New GitHub repo needed.

---

## Success Criteria

### v0.1.0 (Phase 1 + 2)
- Can visually build a 5-qubit circuit via drag-and-drop
- Simulation results display in real-time
- Save/load circuits as JSON
- 5 example circuits included
- Cross-platform builds (Windows, macOS, Linux)

### v0.5.0 (Phase 3)
- Bloch sphere visualization working
- Step-through mode functional
- Entanglement visualization
- Used by at least one university quantum computing course (validation goal)

### v1.0.0 (Phase 4)
- Export to Qiskit, QASM, and LaTeX
- Import from QASM
- 15+ example circuits
- Stable API for ket-core crate
- Published on crates.io

---

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| QuantRS2 too immature | Medium | Custom state vector sim as fallback (~300 LOC) |
| egui 3D rendering for Bloch sphere | Medium | Use 2D projection (circle + vector), skip true 3D |
| Scope creep into full IDE | High | Strict phasing; Ket is a circuit tool, not a quantum IDE |
| Low adoption in niche market | Medium | Target education first; professors need visual tools |
| Name collision | Low | Search before publishing; have backup names |
