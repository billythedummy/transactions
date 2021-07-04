use csv::StringRecord;

use super::Cents;

fn check_eq(ser: &str, de: u64) {
    let record = StringRecord::from(vec![ser]);
    let cents: Cents = record.deserialize(None).unwrap();
    assert_eq!(de, cents.value());
}

fn check_err(ser: &str) {
    let record = StringRecord::from(vec![ser]);
    assert!(record.deserialize::<Cents>(None).is_err());
}

#[test]
fn whole() {
    check_eq("69", 690000);
    check_eq("23435643524", 234356435240000);
}

#[test]
fn decimal() {
    check_eq("1234.1", 12341000);
    check_eq("2432.82", 24328200);
    check_eq("123213.123", 1232131230);
    check_eq("32435532.2435", 324355322435)
}

#[test]
fn too_many_dp() {
    check_err("1234.56789");
}

#[test]
fn malformed_input() {
    check_err("1.23.134");
    check_err(".134");
    check_err("a123");
    check_err("0x123");
}

#[test]
fn display() {
    let cents = Cents::new(12345);
    assert_eq!("1.2345", format!("{}", cents));
    let cents = Cents::new(12000);
    assert_eq!("1.2000", format!("{}", cents));
    let cents = Cents::new(69);
    assert_eq!("0.0069", format!("{}", cents));
    let cents = Cents::new(690);
    assert_eq!("0.0690", format!("{}", cents));
    let cents = Cents::new(100234);
    assert_eq!("10.0234", format!("{}", cents));
    let cents = Cents::new(10000);
    assert_eq!("1.0000", format!("{}", cents));
}
