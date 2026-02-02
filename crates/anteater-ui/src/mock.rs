//! Mock implementations of DebugSession for UI development
//!
//! This module provides realistic mock data for testing UI panels before
//! the real semantic layer is implemented.

use anteater_ui_types::*;
use std::ops::Range;

/// Mock debug session for UI testing
pub struct MockDebugSession {
    status: ExecutionStatus,
    frames: Vec<StackFrame>,
    variables: Vec<Variable>,
    registers: RegisterSet,
    breakpoints: Vec<Breakpoint>,
    memory: Vec<u8>, // Mock memory buffer
}

impl MockDebugSession {
    pub fn new() -> Self {
        Self {
            status: ExecutionStatus::Stopped {
                reason: StopReason::Breakpoint {
                    id: BreakpointId(1),
                },
            },
            frames: create_mock_frames(),
            variables: create_mock_variables(),
            registers: create_mock_registers(),
            breakpoints: create_mock_breakpoints(),
            memory: create_mock_memory(),
        }
    }

    pub fn status(&self) -> ExecutionStatus {
        self.status.clone()
    }

    pub fn stack_frames(&self) -> &[StackFrame] {
        &self.frames
    }

    pub fn variables(&self, _frame: FrameId) -> &[Variable] {
        &self.variables
    }

    pub fn memory(&self, range: Range<u64>) -> Option<&[u8]> {
        let start = range.start as usize;
        let end = range.end.min(self.memory.len() as u64) as usize;

        if start >= self.memory.len() {
            return None;
        }

        Some(&self.memory[start..end])
    }

    pub fn registers(&self) -> &RegisterSet {
        &self.registers
    }

    pub fn disassembly(&self, _range: Range<u64>) -> Vec<DisassemblyLine> {
        create_mock_disassembly()
    }

    pub fn breakpoints(&self) -> &[Breakpoint] {
        &self.breakpoints
    }

    pub fn source_file(&self, _path: &SourcePath) -> Option<SourceFile> {
        Some(create_mock_source_file())
    }

    pub fn current_source_location(&self) -> Option<SourceLocation> {
        Some(SourceLocation {
            path: SourcePath(std::path::PathBuf::from("src/main.rs")),
            line: 42,
            column: Some(8),
        })
    }
}

impl Default for MockDebugSession {
    fn default() -> Self {
        Self::new()
    }
}

fn create_mock_memory() -> Vec<u8> {
    // Create realistic-looking memory with strings, numbers, and patterns
    let mut mem = vec![0u8; 4096];

    // Some string data
    let hello = b"Hello, Anteater!";
    mem[0x100..0x100 + hello.len()].copy_from_slice(hello);

    // Some numeric patterns
    for i in 0..256 {
        mem[0x200 + i] = i as u8;
    }

    // Some stack-like data
    for i in 0..64 {
        let offset = 0x400 + i * 8;
        let value = 0x7fff_0000_0000u64 + (i as u64 * 0x10);
        mem[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
    }

    // Some "random" data
    for i in 0..512 {
        mem[0x800 + i] = ((i * 13 + 7) % 256) as u8;
    }

    mem
}

fn create_mock_source_file() -> SourceFile {
    let source_code = r#"fn main() {
    println!("Starting Anteater example...");

    let data = vec![1, 2, 3, 4, 5];
    let config = Config::default();

    process(data, &config);
}

fn process(mut data: Vec<u8>, config: &Config) {
    println!("Processing {} bytes", data.len());

    for item in &mut data {
        *item *= 2;
    }

    match config.mode {
        Mode::Fast => fast_process(&data),
        Mode::Accurate => accurate_process(&data),
    }
}

fn fast_process(data: &[u8]) {
    // Fast implementation
    for byte in data {
        // Process byte
    }
}

fn accurate_process(data: &[u8]) {
    // Accurate implementation
    for (i, byte) in data.iter().enumerate() {
        // Process byte with index
    }
}

#[derive(Default)]
struct Config {
    mode: Mode,
}

enum Mode {
    Fast,
    Accurate,
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Fast
    }
}"#;

    SourceFile {
        path: SourcePath(std::path::PathBuf::from("src/main.rs")),
        lines: source_code.lines().map(|s| s.to_string()).collect(),
        breakpoints: vec![10, 42],
        executable_lines: vec![1, 2, 4, 5, 7, 10, 11, 13, 14, 15, 17, 18, 19, 23, 24, 25, 26, 31, 32, 33, 34, 42],
    }
}

fn create_mock_frames() -> Vec<StackFrame> {
    vec![
        StackFrame {
            id: FrameId(0),
            function_name: "process::<Vec<u8>>".to_string(),
            source_location: Some(SourceLocation {
                path: SourcePath(std::path::PathBuf::from("src/main.rs")),
                line: 42,
                column: Some(8),
            }),
            instruction_pointer: 0x401234,
            is_inlined: false,
            module: Some("anteater_example".to_string()),
        },
        StackFrame {
            id: FrameId(1),
            function_name: "main".to_string(),
            source_location: Some(SourceLocation {
                path: SourcePath(std::path::PathBuf::from("src/main.rs")),
                line: 10,
                column: Some(4),
            }),
            instruction_pointer: 0x401000,
            is_inlined: false,
            module: Some("anteater_example".to_string()),
        },
        StackFrame {
            id: FrameId(2),
            function_name: "std::rt::lang_start".to_string(),
            source_location: None,
            instruction_pointer: 0x7fff12345678,
            is_inlined: false,
            module: Some("std".to_string()),
        },
    ]
}

fn create_mock_variables() -> Vec<Variable> {
    vec![
        Variable {
            name: "data".to_string(),
            rust_type: TypeDisplay::new("Vec<u8>"),
            value: ValueDisplay::new("Vec(len=3, cap=8)"),
            ownership: OwnershipState::Owned,
            children: vec![
                Variable {
                    name: "len".to_string(),
                    rust_type: TypeDisplay::new("usize"),
                    value: ValueDisplay::new("3"),
                    ownership: OwnershipState::Owned,
                    children: vec![],
                    address: Some(0x7fff_1234_5678),
                    is_optimized_out: false,
                    enum_variant: None,
                },
                Variable {
                    name: "capacity".to_string(),
                    rust_type: TypeDisplay::new("usize"),
                    value: ValueDisplay::new("8"),
                    ownership: OwnershipState::Owned,
                    children: vec![],
                    address: Some(0x7fff_1234_5680),
                    is_optimized_out: false,
                    enum_variant: None,
                },
                Variable {
                    name: "ptr".to_string(),
                    rust_type: TypeDisplay::new("*mut u8"),
                    value: ValueDisplay::new("0x600000000000"),
                    ownership: OwnershipState::Owned,
                    children: vec![],
                    address: Some(0x7fff_1234_5688),
                    is_optimized_out: false,
                    enum_variant: None,
                },
            ],
            address: Some(0x7fff_1234_5678),
            is_optimized_out: false,
            enum_variant: None,
        },
        Variable {
            name: "config".to_string(),
            rust_type: TypeDisplay::new("&Config"),
            value: ValueDisplay::new("0x7fff_1234_5700"),
            ownership: OwnershipState::Borrowed {
                by: vec!["settings".to_string()],
            },
            children: vec![],
            address: Some(0x7fff_1234_5690),
            is_optimized_out: false,
            enum_variant: None,
        },
        Variable {
            name: "old_data".to_string(),
            rust_type: TypeDisplay::new("Vec<u8>"),
            value: ValueDisplay::new("(moved)"),
            ownership: OwnershipState::MovedFrom {
                moved_to: Some("data".to_string()),
            },
            children: vec![],
            address: Some(0x7fff_1234_5698),
            is_optimized_out: false,
            enum_variant: None,
        },
        Variable {
            name: "buffer".to_string(),
            rust_type: TypeDisplay::new("&mut [u8]"),
            value: ValueDisplay::new("[0, 0, 0, 0]"),
            ownership: OwnershipState::MutablyBorrowed {
                by: "writer".to_string(),
            },
            children: vec![],
            address: Some(0x7fff_1234_56a0),
            is_optimized_out: false,
            enum_variant: None,
        },
        Variable {
            name: "result".to_string(),
            rust_type: TypeDisplay::new("Option<i32>"),
            value: ValueDisplay::new("Some(42)"),
            ownership: OwnershipState::Owned,
            children: vec![],
            address: Some(0x7fff_1234_56b0),
            is_optimized_out: false,
            enum_variant: Some("Some".to_string()),
        },
        Variable {
            name: "optimized".to_string(),
            rust_type: TypeDisplay::new("i32"),
            value: ValueDisplay::new("(optimized out)"),
            ownership: OwnershipState::Unknown {
                reason: Some("Optimized out by compiler".to_string()),
            },
            children: vec![],
            address: None,
            is_optimized_out: true,
            enum_variant: None,
        },
    ]
}

fn create_mock_registers() -> RegisterSet {
    RegisterSet {
        general: vec![
            Register {
                name: "rax".to_string(),
                value: 0x0000_0000_0000_002a,
                flags: None,
                changed: true,
            },
            Register {
                name: "rbx".to_string(),
                value: 0x7fff_1234_5678,
                flags: None,
                changed: false,
            },
            Register {
                name: "rcx".to_string(),
                value: 0x0000_0000_0000_0003,
                flags: None,
                changed: false,
            },
            Register {
                name: "rdx".to_string(),
                value: 0x0000_0000_0000_0008,
                flags: None,
                changed: false,
            },
            Register {
                name: "rsi".to_string(),
                value: 0x7fff_1234_5678,
                flags: None,
                changed: false,
            },
            Register {
                name: "rdi".to_string(),
                value: 0x6000_0000_0000,
                flags: None,
                changed: false,
            },
            Register {
                name: "rbp".to_string(),
                value: 0x7fff_ffff_e000,
                flags: None,
                changed: false,
            },
            Register {
                name: "rsp".to_string(),
                value: 0x7fff_ffff_dfe0,
                flags: None,
                changed: false,
            },
            Register {
                name: "rip".to_string(),
                value: 0x0000_0000_0040_1234,
                flags: None,
                changed: true,
            },
        ],
        floating: vec![],
        special: vec![Register {
            name: "rflags".to_string(),
            value: 0x0000_0000_0000_0246,
            flags: Some(vec![
                ("CF".to_string(), false),
                ("ZF".to_string(), true),
                ("SF".to_string(), false),
                ("OF".to_string(), false),
            ]),
            changed: false,
        }],
    }
}

fn create_mock_breakpoints() -> Vec<Breakpoint> {
    vec![
        Breakpoint {
            id: BreakpointId(1),
            enabled: true,
            location: BreakpointLocation::Source {
                location: SourceLocation {
                    path: SourcePath(std::path::PathBuf::from("src/main.rs")),
                    line: 42,
                    column: None,
                },
            },
            condition: None,
            hit_count: 1,
            ignore_count: 0,
        },
        Breakpoint {
            id: BreakpointId(2),
            enabled: false,
            location: BreakpointLocation::Function {
                name: "process".to_string(),
            },
            condition: Some("data.len() > 10".to_string()),
            hit_count: 0,
            ignore_count: 0,
        },
    ]
}

fn create_mock_disassembly() -> Vec<DisassemblyLine> {
    vec![
        DisassemblyLine {
            address: 0x401230,
            bytes: vec![0x55],
            instruction: "push rbp".to_string(),
            source_location: Some(SourceLocation {
                path: SourcePath(std::path::PathBuf::from("src/main.rs")),
                line: 42,
                column: None,
            }),
            is_current: false,
            has_breakpoint: false,
            call_target: None,
        },
        DisassemblyLine {
            address: 0x401231,
            bytes: vec![0x48, 0x89, 0xe5],
            instruction: "mov rbp, rsp".to_string(),
            source_location: None,
            is_current: false,
            has_breakpoint: false,
            call_target: None,
        },
        DisassemblyLine {
            address: 0x401234,
            bytes: vec![0x48, 0x8b, 0x45, 0xf8],
            instruction: "mov rax, [rbp-8]".to_string(),
            source_location: Some(SourceLocation {
                path: SourcePath(std::path::PathBuf::from("src/main.rs")),
                line: 42,
                column: Some(8),
            }),
            is_current: true,
            has_breakpoint: true,
            call_target: None,
        },
        DisassemblyLine {
            address: 0x401238,
            bytes: vec![0xe8, 0xc3, 0x00, 0x00, 0x00],
            instruction: "call 0x401300".to_string(),
            source_location: None,
            is_current: false,
            has_breakpoint: false,
            call_target: Some("transform".to_string()),
        },
    ]
}
