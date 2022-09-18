

pub fn rueppel(n:usize) -> i128 {
    if (n+1).is_power_of_two() {
        1
    } else {
        0
    }
}

pub fn rook(mut n:usize) -> i128 {
    if n == 0 {
        return 0;
    }
    while n % 2 == 0 {
        n = n >> 1
    }
    return ((n>>1) % 2) as i128;
}
pub fn knight(n:usize) -> i128 {
    let rook_n_minus_1 = if n == 0 {
        1
    } else {
        rook(n-1)
    };

    (( rook(n+1) - rook_n_minus_1 ).rem_euclid(2) ) as i128
}
pub fn pagoda(n:usize) -> i128 {
	let rook_n_minus_1 = if n == 0 {
		1
	} else {
		rook(n-1)
	};

	(( rook(n+1) - rook_n_minus_1 ).rem_euclid(3) ) as i128
}

#[derive(Debug, Clone, Copy)]
enum ZigZagSequenceItem {
	A, B, C, D, E, F
}
impl ZigZagSequenceItem {
	fn expand(self) -> [Self; 3] {
		match self {
			Self::A => [Self::A, Self::C, Self::B],
			Self::B => [Self::B, Self::C, Self::B],
			Self::C => [Self::E, Self::D, Self::F],
			Self::D => [Self::D, Self::D, Self::D],
			Self::E => [Self::E, Self::D, Self::D],
			Self::F => [Self::D, Self::D, Self::F]
		}
	}

	fn value(self) -> i128 {
		match self {
			Self::A => 1,
			Self::B => 0,
			Self::C => 1,
			Self::D => 0,
			Self::E => 2,
			Self::F => 2
		}
	}
}
fn get_zigzag_position(position: usize) -> ZigZagSequenceItem {
	if position == 0 { return ZigZagSequenceItem::A; };
	if position == 1 { return ZigZagSequenceItem::C; };
	if position == 2 { return ZigZagSequenceItem::B; };

	let expand_position = position.rem_euclid(3);

	let base_position = position-expand_position;

	let previous_position = base_position/3;
	get_zigzag_position(previous_position).expand()[expand_position]
}
pub fn zigzag(n:usize) -> i128 {
	get_zigzag_position(n as usize).value()
}