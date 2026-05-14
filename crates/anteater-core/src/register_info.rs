use nix::libc;

pub struct RegisterInfo {
    pub id: RegisterId,
    pub name: &'static str,
    pub dwarf_id: Option<u32>,
    pub size: usize,
    pub offset: usize,
    pub kind: RegisterType,
    pub format: RegisterFormat,
}

pub enum RegisterType {
    Gpr,
    SubGpr,
    Fpr,
    Dr,
}

pub enum RegisterFormat {
    Uint,
    DoubleFloat,
    LongDouble,
    Vector,
}

macro_rules! define_registers {
    (
    $( gpr_64 { $name:ident = $d:literal } ),* $(,)?
    ) => {
        #[allow(non_camel_case_types)]
        pub enum RegisterId {
            $( $name, )*
        }

        pub static REGISTER_INFOS: &[RegisterInfo] = &[
            $(
            RegisterInfo {
                id: RegisterId::$name,
                name: stringify!($name),
                dwarf_id: Some($d),
                size: 8,
                offset: core::mem::offset_of!(libc::user, regs.$name),
                kind: RegisterType::Gpr,
                format: RegisterFormat::Uint,
            },
            )*
        ];
    }
}

define_registers! {
    gpr_64 { rax = 0 },
    gpr_64 { rdx = 1},
    // gpr_32 { Eax => Rax},
    // gpr_16 { Ax => Rax },
    // commenting out so we can work with one branch to start.
}

// 3. Let the compiler tell you what's missing. `RegisterInfo` undefined?
//    Add it. `RegisterType::Gpr` undefined? Add the enum. The compiler is
//    your TODO list — don't pre-write types you don't yet need.
//
// 4. Once one register builds clean, write a smoke test:
//        assert_eq!(REGISTER_INFOS[0].name, "rax");
//
// 5. Add the remaining arms (gpr32, gpr16, gpr8h, gpr8l, fpr, fp_st,
//    fp_mm, fp_xmm, dr) one at a time. Add a few rows per arm; rebuild;
//    move on. The full row list lives in:
//        sdb/include/libsdb/detail/registers.inc   (chapter-5 branch)
//
// 6. Lookup fns last — trivial `.iter().find(...)` over `REGISTER_INFOS`.
//
// =====================================================================
// Decisions to make BEFORE writing the macro body
// =====================================================================
//
// - dwarf_id type: `i32` (1:1 with C++ -1 sentinel) or `Option<u32>`
//   (idiomatic Rust)? Affects the macro's input shape.
// - Lookup miss semantics: `Option<&RegisterInfo>` (caller decides),
//   `Result<&RegisterInfo>` using your `Error` enum (matches Sy's throw),
//   or panic on by_id (it's an enum, every variant must be in the table).
//
// =====================================================================
// Helpers you'll need inside the macro
// =====================================================================
//
// - `core::mem::offset_of!` (stable since 1.77) for `offsetof(user, regs.rax)`.
// - `libc::user`, `libc::user_regs_struct`, `libc::user_fpregs_struct`
//   for the field layouts. Add `libc` to Cargo.toml.
//
// =====================================================================
// Does NOT belong here
// =====================================================================
//
//   - The `Registers` struct (live process snapshot) → `registers.rs`
//   - Anything that calls ptrace                     → `registers.rs`
//   - Anything that touches a `Process`              → `registers.rs`
