use serde_env_field::EnvField;

#[test]
fn test_eq() {
    let field: EnvField<i32> = 10.into();

    // assert_eq!(10, field); -- can't implement

    assert_eq!(field, 10);
    assert_eq!(10, *field);
}

#[test]
fn test_ord() {
    let field: EnvField<i32> = 10.into();

    // assert!(<num> <op> field); -- can't implement

    assert!(field > 9);
    assert!(9 < *field);

    assert!(field >= 9);
    assert!(9 <= *field);

    assert!(field < 11);
    assert!(11 > *field);

    assert!(field <= 10);
    assert!(10 >= *field);
}

#[test]
fn test_eq_str() {
    let field: EnvField<String> = "test".to_string().into();

    assert_eq!(&field, "test");
}

#[test]
fn test_add() {
    let field: EnvField<i32> = 10.into();

    assert_eq!(field + 12, 22);
    assert_eq!(field + field, 20);
}

#[test]
fn test_sub() {
    let field: EnvField<i32> = 10.into();

    assert_eq!(field - 12, -2);
    assert_eq!(field - field, 0);
}

#[test]
fn test_mul() {
    let field: EnvField<i32> = 10.into();

    assert_eq!(field * 12, 120);
    assert_eq!(field * field, 100);
}

#[test]
fn test_div() {
    let field: EnvField<i32> = 10.into();

    assert_eq!(field / 12, 0);
    assert_eq!(field / field, 1);
}

#[test]
fn test_rem() {
    let field: EnvField<i32> = 10.into();

    assert_eq!(field % 12, 10);
    assert_eq!(field % field, 0);
}

#[test]
fn test_neg() {
    let field: EnvField<i32> = 10.into();

    assert_eq!(-field, -10);
}

#[test]
fn test_bit_and() {
    let field: EnvField<i32> = 0xA.into();

    assert_eq!(field & 0x2, 0x2);
    assert_eq!(field & field, 0xA);
}

#[test]
fn test_bit_or() {
    let field: EnvField<i32> = 0xA.into();

    assert_eq!(field | 0x3, 0xB);
    assert_eq!(field | field, 0xA);
}

#[test]
fn test_bit_xor() {
    let field: EnvField<i32> = 0xA.into();

    assert_eq!(field ^ 0x2, 0x8);
    assert_eq!(field ^ field, 0x0);
}

#[test]
fn test_shl() {
    let field: EnvField<i32> = 0xA.into();

    assert_eq!(field << 1, 0x14);
    assert_eq!(field << field, 0x2800);
}

#[test]
fn test_shr() {
    let field: EnvField<i32> = 0xA.into();

    assert_eq!(field >> 1, 0x5);
    assert_eq!(field >> field, 0x0);
}

#[test]
fn test_bit_not() {
    let field: EnvField<i32> = 0xA.into();

    assert_eq!(!field, -0xB);
}

#[test]
fn test_add_assign() {
    let mut field: EnvField<i32> = 10.into();
    field += field;
    field += 12;

    assert_eq!(field, 32);
}

#[test]
fn test_sub_assign() {
    let mut field: EnvField<i32> = 10.into();
    field -= field;
    field -= 12;

    assert_eq!(field, -12);
}

#[test]
fn test_mul_assign() {
    let mut field: EnvField<i32> = 10.into();
    field *= field;
    field *= 12;

    assert_eq!(field, 1200);
}

#[test]
fn test_div_assign() {
    let mut field: EnvField<i32> = 10.into();
    field /= field;
    field /= 12;

    assert_eq!(field, 0);
}

#[test]
fn test_rem_assign() {
    let mut field: EnvField<i32> = 10.into();
    field %= field;
    field %= 12;

    assert_eq!(field, 0);
}

#[test]
fn test_bit_and_assign() {
    let mut field: EnvField<i32> = 0xA.into();
    field &= field;
    field &= 0x2;

    assert_eq!(field, 0x2);
}

#[test]
fn test_bit_or_assign() {
    let mut field: EnvField<i32> = 0xA.into();
    field |= field;
    field |= 0x3;

    assert_eq!(field, 0xB);
}

#[test]
fn test_bit_xor_assign() {
    let mut field: EnvField<i32> = 0xA.into();
    field ^= field;
    field ^= 0x8;

    assert_eq!(field, 0x8);
}

#[test]
fn test_shl_assign() {
    let mut field: EnvField<i32> = 0xA.into();
    field <<= field;
    field <<= 1;

    assert_eq!(field, 0x5000);
}

#[test]
fn test_shr_assign() {
    let mut field: EnvField<i32> = 0xA.into();
    field >>= field;
    field >>= 1;

    assert_eq!(field, 0x0);
}
