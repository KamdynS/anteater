use crate::error::{Error, Result};
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
    // base
    (@go [$($variants:tt)*] [$($entries:tt)*]) => {
        #[allow(non_camel_case_types)]
        #[derive(PartialEq, Eq, Copy, Clone, Debug)]
        pub enum RegisterId {
            $($variants)*
        }

        pub static REGISTER_INFOS: &[RegisterInfo] = &[
            $($entries)*
        ];
    };

    // gpr_64 with dwarf id
    (@go
        [$($variants:tt)*]
        [$($entries:tt)*]
        gpr_64 { $name:ident = $d:literal } , $($rest:tt)*
    ) => {
        define_registers!(@go
            [$($variants)* $name,]
            [$($entries)*
                RegisterInfo {
                    id: RegisterId::$name,
                    name: stringify!($name),
                    dwarf_id: Some($d),
                    size: 8,
                    offset: core::mem::offset_of!(libc::user, regs.$name),
                    kind: RegisterType::Gpr,
                    format: RegisterFormat::Uint,
                },
            ]
            $($rest)*
        );
    };

    // gpr_64 without dwarf id
    (@go
        [$($variants:tt)*]
        [$($entries:tt)*]
        gpr_64 { $name:ident } , $($rest:tt)*
    ) => {
        define_registers!(@go
            [$($variants)* $name,]
            [$($entries)*
                RegisterInfo {
                    id: RegisterId::$name,
                    name: stringify!($name),
                    dwarf_id: None,
                    size: 8,
                    offset: core::mem::offset_of!(libc::user, regs.$name),
                    kind: RegisterType::Gpr,
                    format: RegisterFormat::Uint,
                },
            ]
            $($rest)*
        );
    };

    // gpr_32
    (@go
        [$($variants:tt)*]
        [$($entries:tt)*]
        gpr_32 { $name:ident => $super:ident } , $($rest:tt)*
    ) => {
        define_registers!(@go
            [$($variants)* $name,]
            [$($entries)*
                RegisterInfo {
                    id: RegisterId::$name,
                    name: stringify!($name),
                    dwarf_id: None,
                    size: 4,
                    offset: core::mem::offset_of!(libc::user, regs.$super),
                    kind: RegisterType::SubGpr,
                    format: RegisterFormat::Uint,
                },
            ]
            $($rest)*
        );
    };

    // gpr_16
    (@go
        [$($variants:tt)*]
        [$($entries:tt)*]
        gpr_16 { $name:ident => $super:ident } , $($rest:tt)*
    ) => {
        define_registers!(@go
            [$($variants)* $name,]
            [$($entries)*
                RegisterInfo {
                    id: RegisterId::$name,
                    name: stringify!($name),
                    dwarf_id: None,
                    size: 2,
                    offset: core::mem::offset_of!(libc::user, regs.$super),
                    kind: RegisterType::SubGpr,
                    format: RegisterFormat::Uint,
                },
            ]
            $($rest)*
        );
    };

    // gpr_8h — high byte of low 16, offset + 1
    (@go
        [$($variants:tt)*]
        [$($entries:tt)*]
        gpr_8h { $name:ident => $super:ident } , $($rest:tt)*
    ) => {
        define_registers!(@go
            [$($variants)* $name,]
            [$($entries)*
                RegisterInfo {
                    id: RegisterId::$name,
                    name: stringify!($name),
                    dwarf_id: None,
                    size: 1,
                    offset: core::mem::offset_of!(libc::user, regs.$super) + 1,
                    kind: RegisterType::SubGpr,
                    format: RegisterFormat::Uint,
                },
            ]
            $($rest)*
        );
    };

    // gpr_8l — low byte
    (@go
        [$($variants:tt)*]
        [$($entries:tt)*]
        gpr_8l { $name:ident => $super:ident } , $($rest:tt)*
    ) => {
        define_registers!(@go
            [$($variants)* $name,]
            [$($entries)*
                RegisterInfo {
                    id: RegisterId::$name,
                    name: stringify!($name),
                    dwarf_id: None,
                    size: 1,
                    offset: core::mem::offset_of!(libc::user, regs.$super),
                    kind: RegisterType::SubGpr,
                    format: RegisterFormat::Uint,
                },
            ]
            $($rest)*
        );
    };

    // fpr with dwarf id (named field in i387, explicit size)
    (@go
        [$($variants:tt)*]
        [$($entries:tt)*]
        fpr { $name:ident = $d:literal => $field:ident : $size:literal } , $($rest:tt)*
    ) => {
        define_registers!(@go
            [$($variants)* $name,]
            [$($entries)*
                RegisterInfo {
                    id: RegisterId::$name,
                    name: stringify!($name),
                    dwarf_id: Some($d),
                    size: $size,
                    offset: core::mem::offset_of!(libc::user, i387.$field),
                    kind: RegisterType::Fpr,
                    format: RegisterFormat::Uint,
                },
            ]
            $($rest)*
        );
    };

    // fpr without dwarf id
    (@go
        [$($variants:tt)*]
        [$($entries:tt)*]
        fpr { $name:ident => $field:ident : $size:literal } , $($rest:tt)*
    ) => {
        define_registers!(@go
            [$($variants)* $name,]
            [$($entries)*
                RegisterInfo {
                    id: RegisterId::$name,
                    name: stringify!($name),
                    dwarf_id: None,
                    size: $size,
                    offset: core::mem::offset_of!(libc::user, i387.$field),
                    kind: RegisterType::Fpr,
                    format: RegisterFormat::Uint,
                },
            ]
            $($rest)*
        );
    };

    // fp_st — stN, 16 bytes, st_space + idx*16, long_double
    (@go
        [$($variants:tt)*]
        [$($entries:tt)*]
        fp_st { $name:ident = $d:literal , $idx:literal } , $($rest:tt)*
    ) => {
        define_registers!(@go
            [$($variants)* $name,]
            [$($entries)*
                RegisterInfo {
                    id: RegisterId::$name,
                    name: stringify!($name),
                    dwarf_id: Some($d),
                    size: 16,
                    offset: core::mem::offset_of!(libc::user, i387.st_space) + $idx * 16,
                    kind: RegisterType::Fpr,
                    format: RegisterFormat::LongDouble,
                },
            ]
            $($rest)*
        );
    };

    // fp_mm — mmN, 8 bytes, st_space + idx*16, vector
    (@go
        [$($variants:tt)*]
        [$($entries:tt)*]
        fp_mm { $name:ident = $d:literal , $idx:literal } , $($rest:tt)*
    ) => {
        define_registers!(@go
            [$($variants)* $name,]
            [$($entries)*
                RegisterInfo {
                    id: RegisterId::$name,
                    name: stringify!($name),
                    dwarf_id: Some($d),
                    size: 8,
                    offset: core::mem::offset_of!(libc::user, i387.st_space) + $idx * 16,
                    kind: RegisterType::Fpr,
                    format: RegisterFormat::Vector,
                },
            ]
            $($rest)*
        );
    };

    // fp_xmm — xmmN, 16 bytes, xmm_space + idx*16, vector
    (@go
        [$($variants:tt)*]
        [$($entries:tt)*]
        fp_xmm { $name:ident = $d:literal , $idx:literal } , $($rest:tt)*
    ) => {
        define_registers!(@go
            [$($variants)* $name,]
            [$($entries)*
                RegisterInfo {
                    id: RegisterId::$name,
                    name: stringify!($name),
                    dwarf_id: Some($d),
                    size: 16,
                    offset: core::mem::offset_of!(libc::user, i387.xmm_space) + $idx * 16,
                    kind: RegisterType::Fpr,
                    format: RegisterFormat::Vector,
                },
            ]
            $($rest)*
        );
    };

    // dr — drN, no dwarf, 8 bytes, u_debugreg + idx*8
    (@go
        [$($variants:tt)*]
        [$($entries:tt)*]
        dr { $name:ident , $idx:literal } , $($rest:tt)*
    ) => {
        define_registers!(@go
            [$($variants)* $name,]
            [$($entries)*
                RegisterInfo {
                    id: RegisterId::$name,
                    name: stringify!($name),
                    dwarf_id: None,
                    size: 8,
                    offset: core::mem::offset_of!(libc::user, u_debugreg) + $idx * 8,
                    kind: RegisterType::Dr,
                    format: RegisterFormat::Uint,
                },
            ]
            $($rest)*
        );
    };

    // unknown @go input — fail loudly instead of falling through to entry and looping
    (@go [$($variants:tt)*] [$($entries:tt)*] $unknown:tt $($rest:tt)*) => {
        compile_error!(concat!(
            "define_registers!: unknown row kind near `",
            stringify!($unknown),
            "`"
        ));
    };

    // entry — must come LAST: the catch-all $($input:tt)* would otherwise
    // re-match every recursive @go call and infinitely re-wrap them.
    ( $($input:tt)* ) => {
        define_registers!(@go [] [] $($input)*);
    };
}

define_registers! {
    gpr_64 { rax = 0 },
    gpr_64 { rdx = 1 },
    gpr_64 { rcx = 2 },
    gpr_64 { rbx = 3 },
    gpr_64 { rsi = 4 },
    gpr_64 { rdi = 5 },
    gpr_64 { rbp = 6 },
    gpr_64 { rsp = 7 },
    gpr_64 { r8 = 8 },
    gpr_64 { r9 = 9 },
    gpr_64 { r10 = 10 },
    gpr_64 { r11 = 11 },
    gpr_64 { r12 = 12 },
    gpr_64 { r13 = 13 },
    gpr_64 { r14 = 14 },
    gpr_64 { r15 = 15 },
    gpr_64 { rip = 16 },
    gpr_64 { eflags = 49 },
    gpr_64 { cs = 51 },
    gpr_64 { fs = 54 },
    gpr_64 { gs = 55 },
    gpr_64 { ss = 52 },
    gpr_64 { ds = 53 },
    gpr_64 { es = 50 },
    gpr_64 { orig_rax },

    gpr_32 { eax => rax },
    gpr_32 { edx => rdx },
    gpr_32 { ecx => rcx },
    gpr_32 { ebx => rbx },
    gpr_32 { esi => rsi },
    gpr_32 { edi => rdi },
    gpr_32 { ebp => rbp },
    gpr_32 { esp => rsp },
    gpr_32 { r8d => r8 },
    gpr_32 { r9d => r9 },
    gpr_32 { r10d => r10 },
    gpr_32 { r11d => r11 },
    gpr_32 { r12d => r12 },
    gpr_32 { r13d => r13 },
    gpr_32 { r14d => r14 },
    gpr_32 { r15d => r15 },

    gpr_16 { ax => rax },
    gpr_16 { dx => rdx },
    gpr_16 { cx => rcx },
    gpr_16 { bx => rbx },
    gpr_16 { si => rsi },
    gpr_16 { di => rdi },
    gpr_16 { bp => rbp },
    gpr_16 { sp => rsp },
    gpr_16 { r8w => r8 },
    gpr_16 { r9w => r9 },
    gpr_16 { r10w => r10 },
    gpr_16 { r11w => r11 },
    gpr_16 { r12w => r12 },
    gpr_16 { r13w => r13 },
    gpr_16 { r14w => r14 },
    gpr_16 { r15w => r15 },

    gpr_8h { ah => rax },
    gpr_8h { dh => rdx },
    gpr_8h { ch => rcx },
    gpr_8h { bh => rbx },

    gpr_8l { al => rax },
    gpr_8l { dl => rdx },
    gpr_8l { cl => rcx },
    gpr_8l { bl => rbx },
    gpr_8l { sil => rsi },
    gpr_8l { dil => rdi },
    gpr_8l { bpl => rbp },
    gpr_8l { spl => rsp },
    gpr_8l { r8b => r8 },
    gpr_8l { r9b => r9 },
    gpr_8l { r10b => r10 },
    gpr_8l { r11b => r11 },
    gpr_8l { r12b => r12 },
    gpr_8l { r13b => r13 },
    gpr_8l { r14b => r14 },
    gpr_8l { r15b => r15 },

    fpr { fcw = 65 => cwd : 2 },
    fpr { fsw = 66 => swd : 2 },
    fpr { ftw => ftw : 2 },
    fpr { fop => fop : 2 },
    fpr { frip => rip : 8 },
    fpr { frdp => rdp : 8 },
    fpr { mxcsr = 64 => mxcsr : 4 },
    fpr { mxcsrmask => mxcr_mask : 4 },

    fp_st { st0 = 33, 0 },
    fp_st { st1 = 34, 1 },
    fp_st { st2 = 35, 2 },
    fp_st { st3 = 36, 3 },
    fp_st { st4 = 37, 4 },
    fp_st { st5 = 38, 5 },
    fp_st { st6 = 39, 6 },
    fp_st { st7 = 40, 7 },

    fp_mm { mm0 = 41, 0 },
    fp_mm { mm1 = 42, 1 },
    fp_mm { mm2 = 43, 2 },
    fp_mm { mm3 = 44, 3 },
    fp_mm { mm4 = 45, 4 },
    fp_mm { mm5 = 46, 5 },
    fp_mm { mm6 = 47, 6 },
    fp_mm { mm7 = 48, 7 },

    fp_xmm { xmm0 = 17, 0 },
    fp_xmm { xmm1 = 18, 1 },
    fp_xmm { xmm2 = 19, 2 },
    fp_xmm { xmm3 = 20, 3 },
    fp_xmm { xmm4 = 21, 4 },
    fp_xmm { xmm5 = 22, 5 },
    fp_xmm { xmm6 = 23, 6 },
    fp_xmm { xmm7 = 24, 7 },
    fp_xmm { xmm8 = 25, 8 },
    fp_xmm { xmm9 = 26, 9 },
    fp_xmm { xmm10 = 27, 10 },
    fp_xmm { xmm11 = 28, 11 },
    fp_xmm { xmm12 = 29, 12 },
    fp_xmm { xmm13 = 30, 13 },
    fp_xmm { xmm14 = 31, 14 },
    fp_xmm { xmm15 = 32, 15 },

    dr { dr0, 0 },
    dr { dr1, 1 },
    dr { dr2, 2 },
    dr { dr3, 3 },
    dr { dr4, 4 },
    dr { dr5, 5 },
    dr { dr6, 6 },
    dr { dr7, 7 },
}

impl RegisterInfo {
    pub fn by_id(id: RegisterId) -> Result<&'static Self> {
        REGISTER_INFOS
            .iter()
            .find(|i| i.id == id)
            .ok_or(Error::NoSuchRegister(format!("{id:?}")))
    }
    pub fn by_name(name: &str) -> Result<&'static Self> {
        REGISTER_INFOS
            .iter()
            .find(|i| i.name == name)
            .ok_or(Error::NoSuchRegister(format!("{name:?}")))
    }
    pub fn by_dwarf(dwarf_id: u32) -> Result<&'static Self> {
        REGISTER_INFOS
            .iter()
            .find(|i| i.dwarf_id == Some(dwarf_id))
            .ok_or(Error::NoSuchRegister(format!("{dwarf_id:?}")))
    }
}
