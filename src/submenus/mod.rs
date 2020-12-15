pub(crate) mod ratio_menu;
pub(crate) mod resolution_menu;

fn calculate_aspect_ratio(x: i32, y: i32) -> (i32, i32) {
    let gcd = num::integer::gcd(y, x);
    (x / gcd, y / gcd)
}
