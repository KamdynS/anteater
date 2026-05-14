// registers.rs — the live, per-process register snapshot.
//
// Mirrors Sy's `registers.hpp` + `registers.cpp`. Consumes the static table
// from `register_info.rs` to read/write a real process's registers via ptrace.
//
// DO NOT START THIS FILE until `register_info.rs` is done and tested.
// Two seams, two steps. This file depends on the table existing.
//
// =====================================================================
// What goes in this file (you write all of it by hand — no macros here)
// =====================================================================
//
// 1. `Value` enum — a typed register value.
//    C++ uses `std::variant` over a bunch of int/float/byte-array types.
//    In Rust:
//        pub enum Value {
//            U8(u8), U16(u16), U32(u32), U64(u64),
//            I8(i8), I16(i16), I32(i32), I64(i64),
//            F32(f32), F64(f64),
//            LongDouble([u8; 16]),   // x87 80-bit, padded to 16
//            Byte64([u8; 8]),        // mm0..mm7
//            Byte128([u8; 16]),      // xmm0..xmm15
//        }
//    Cross-check exact set against Sy's `types.hpp` on the chapter-5 branch.
//
// 2. `Registers` struct — one snapshot of the kernel user-area for a process.
//        pub struct Registers {
//            pub data: libc::user,
//            // possibly: pid: Pid, if read/write need it directly
//        }
//    Fat struct, pub field, no getter. Sy stores a `user` by value plus a
//    back-pointer to the owning process; in Rust either keep `pid` inline
//    here or pass it into read/write from `Process`. Pick one and be
//    consistent.
//
// 3. Methods on `Registers`:
//        pub fn read(&self, info: &RegisterInfo) -> Value
//        pub fn write(&mut self, info: &RegisterInfo, value: Value) -> Result<()>
//
//    `read`:  pull `info.size` bytes from `&self.data` at byte offset
//             `info.offset`, pack into the right `Value` variant per
//             `info.format` + `info.size`.
//    `write`: inverse — unpack `Value`, splat into `self.data` at offset,
//             then push the affected word(s) back to the kernel via ptrace.
//
//    GPRs vs FPRs may take different ptrace paths (PTRACE_PEEKUSER /
//    POKEUSER vs GETREGS/SETREGS / GETFPREGS/SETFPREGS). Check Sy's
//    `registers.cpp` on chapter-5 to see which combination he uses.
//
//    Use `nix::sys::ptrace`. PEEKUSER/POKEUSER are word-granularity, so
//    you'll likely want a small helper that round-trips full words even
//    when writing a sub-register byte.
//
// =====================================================================
// Bridge to `Process` (lives in process.rs, not here)
// =====================================================================
//
// `Process` grows a `pub registers: Registers` field. Sy also adds
// `read_all_registers` / `write_all_registers` on the process itself
// (PTRACE_GETREGS + GETFPREGS to fill the snapshot, the inverses to flush
// it). Those methods live on `Process` in `process.rs` — `Registers` here
// stays inert storage.
//
// =====================================================================
// Does NOT belong here
// =====================================================================
//
//   - REGISTER_INFOS table        → register_info.rs
//   - RegisterId enum             → register_info.rs (macro-generated there)
//   - define_registers! macro     → register_info.rs
