use anteater_core::register_info::*;

#[test]
fn rax_basics() {
    assert_eq!(REGISTER_INFOS.len(), 1);
    let r = &REGISTER_INFOS[0];
    assert_eq!(r.name, "rax");
    assert_eq!(r.dwarf_id, 0);
    assert_eq!(r.size, 8);
    assert!(matches!(r.kind, RegisterType::Gpr));
    assert!(matches!(r.format, RegisterFormat::Uint));
}
