use std::{collections::HashMap, fs};
use std::env;
use json::{
    JsonValue
};
use image;

use number_wall_generator::{
    repeating_sequence_wall, left_const_wall, bi_directional_wall
};

mod left_const_functions;
mod bi_directional_functions;

const DEFAULT_MODULO: i128 = 0;
const DEFAULT_OUTPUT_FILE: &'static str = "./out.png";
const DEFAULT_LEFT_VALUES: [i128; 2] = [0, 0];
const DEFAULT_SEQUENCE_START: isize = 0;

const DEFAULT_COLOURS: [(i128, [u8; 3]); 2] = [
    (0, [255, 255, 255]),
    (1, [0, 0, 0])
];
const DEFAULT_DEFAULT_COLOUR: [u8; 3] = [128, 128, 128];

fn main() {
    let mut args = env::args();

    let config_file = args.nth(1).expect("No argument was given");

    let config = fs::read(config_file).unwrap();

    let input = json::parse(&String::from_utf8(config).unwrap()).unwrap();

    execute_input(&input).unwrap()
}

struct Boundary {
    top: usize,
    bottom: usize,
    left: isize,
    right: isize
}
enum WallType {
    Repeating,
    LeftConst,
    BiDirectional
}
enum Sequence {
    Knight,
    Rook,
    Pagoda,
    Rueppel,
    Zigzag,
    Custom(Vec<i128>)
}

struct Colours {
    mapping: HashMap<i128, [u8; 3]>,
    default: [u8; 3]
}


fn get_wall_type(input: &JsonValue) -> Result<WallType, String> {
    if !input["wall_type"].is_string() {
        return Err("wall_type is not a string".into());
    }
    let wall_type = input["wall_type"].as_str().unwrap();
    let wall_type = wall_type.to_ascii_lowercase();
    if wall_type == "left_const" || wall_type == "leftconst" {
        Ok(WallType::LeftConst)
    } else if wall_type == "bi_directional" || wall_type == "bidirectional" {
        Ok(WallType::BiDirectional)
    } else if wall_type == "repeating" {
        Ok(WallType::Repeating)
    } else {
        Err(format!("{wall_type} is not a valid wall_type"))
    }
}

fn get_sequence(input: &JsonValue) -> Result<Sequence, String> {
    if input["sequence"].is_string() {
        let sequence_name = input["sequence"].as_str().unwrap();
        let sequence_name = sequence_name.to_ascii_lowercase();
        if sequence_name == "knight" {
            Ok(Sequence::Knight)
        } else if sequence_name == "rook" {
            Ok(Sequence::Rook)
        } else if sequence_name == "pagoda" {
            Ok(Sequence::Pagoda)
        } else if sequence_name == "rueppel" {
            Ok(Sequence::Rueppel)
        } else if sequence_name == "zigzag" {
            Ok(Sequence::Zigzag)
        } else {
            Err(format!("{sequence_name} is not a valid sequence"))
        }
    } else if input["sequence"].is_array() {
        let mut sequence: Vec<i128> = vec![];
        for pos in 0..input["sequence"].len() {
            let val = &input["sequence"][pos];
            if !val.is_number() {
                return Err(format!("value at position {pos} in sequence is not a number"));
            };
            let v: i128 = match val.as_i64() {
                Some(s) => s.into(),
                None => { return Err(format!("unexpected error parsing number at position {pos} in sequence")); }
            };
            sequence.push(v);
        };
        Ok(Sequence::Custom(sequence))
    } else {
        Err("sequence is not a string or an array".into())
    }
}

fn get_modulo(input: &JsonValue) -> Result<i128, String> {
    if input["modulo"].is_null() {
        return Ok(DEFAULT_MODULO);
    };
    if !input["modulo"].is_number() {
        return Err("modulo is not a number".into());
    };
    match input["modulo"].as_u64() {
        Some(s) => Ok(s.into()),
        None => Err(format!("error parsing modulo: it must not be negative or above {}", u64::MAX))
    }
}

fn get_left_values(input: &JsonValue) -> Result<[i128; 2], String> {
    let left_values = &input["left_values"];
    if left_values.is_null() {
        return Ok(DEFAULT_LEFT_VALUES);
    };
    if !left_values.is_array() {
        return Err("left_values is not an array".into());
    };
    if left_values.len() != 2 {
        return Err("left_values does not have 2 elements".into());
    };
    if !left_values[0].is_number() {
        return Err(format!("left_values element 0 ({}) is not a number", left_values[0]));
    };
    if !left_values[1].is_number() {
        return Err(format!("left_values element 1 ({}) is not a number", left_values[1]));
    };
    let num_0: i128 = match left_values[0].as_i64() {
        Some(s) => s.into(),
        None => { return Err(format!("Unexpected error parsing left_values[0] ({}), it may be too large", left_values[0])); }
    };
    let num_1: i128 = match left_values[1].as_i64() {
        Some(s) => s.into(),
        None => { return Err(format!("Unexpected error parsing left_values[1] ({}), it may be too large", left_values[1])); }
    };
    Ok([num_0, num_1])
}

fn get_boundary(input: &JsonValue) -> Result<Boundary, String> {
    if !input["top"].is_number() {
        return Err("top is not a number".into());
    };
    let top: usize = match input["top"].as_usize() {
        Some(s) => s,
        None => { return Err(format!("{} is not valid for top, is must be non negative and less than {}", input["top"], usize::MAX)); }
    };

    if !input["bottom"].is_number() {
        return Err("bottom is not a number".into());
    };
    let bottom: usize = match input["bottom"].as_usize() {
        Some(s) => s,
        None => { return Err(format!("{} is not valid for bottom, is must be non negative and less than {}", input["bottom"], usize::MAX)); }
    };

    if !input["left"].is_number() {
        return Err("left is not a number".into());
    };
    let left: isize = match input["left"].as_isize() {
        Some(s) => s,
        None => { return Err(format!("{} is not valid for left, is must be non negative and between {} and {}", input["left"], isize::MIN, isize::MAX)); }
    };

    if !input["right"].is_number() {
        return Err("right is not a number".into());
    };
    let right: isize = match input["right"].as_isize() {
        Some(s) => s,
        None => { return Err(format!("{} is not valid for right, is must be non negative and between {} and {}", input["right"], isize::MIN, isize::MAX)); }
    };

    Ok(Boundary {
        top,
        bottom,
        left,
        right,
    })
}

fn get_output_file(input: &JsonValue) -> Result<String, String> {
    if input["output_file"].is_null() {
        return Ok(DEFAULT_OUTPUT_FILE.into());
    };
    if !input["output_file"].is_string() {
        return Err("output_file is not a string".into());
    };
    return Ok( input["output_file"].as_str().unwrap().to_owned() );
}

fn get_sequence_start(input: &JsonValue) -> Result<isize, String> {
    if input["sequence_start"].is_null() {
        return Ok(DEFAULT_SEQUENCE_START);
    };
    if !input["sequence_start"].is_number() {
        return Err("sequence_start is not a number".into());
    };
    let ss: Result<isize, String> = input["sequence_start"].as_isize().ok_or("Could not parse sequence_start  as a number".into());
    return Ok( ss?.to_owned() );
}

fn colour_to_u8_array(colour: &JsonValue) -> Result<[u8; 3], String> {
    if colour.is_array() {
        if colour.len() != 3 {
            Err(format!("Colour array {colour} doesn't have 3 elements"))
        } else {
            Ok([
                colour[0].as_u8().ok_or(format!("Could not parse element 0 of {colour} as a u8"))?,
                colour[1].as_u8().ok_or(format!("Could not parse element 1 of {colour} as a u8"))?,
                colour[2].as_u8().ok_or(format!("Could not parse element 2 of {colour} as a u8"))?,
            ])
        }
    } else if colour.is_string() {
        let colour = colour.as_str().unwrap();
        if colour.len() == 3 {
            Ok([
                0x10 * u8::from_str_radix(&colour[0..=0], 16).ok().ok_or(format!("Could not parse character 0 of {colour} as a base 16 value") )?,
                0x10 * u8::from_str_radix(&colour[1..=1], 16).ok().ok_or(format!("Could not parse character 1 of {colour} as a base 16 value") )?,
                0x10 * u8::from_str_radix(&colour[2..=2], 16).ok().ok_or(format!("Could not parse character 2 of {colour} as a base 16 value") )?,
            ])
        } else if colour.len() == 4 {
            Ok([
                0x10 * u8::from_str_radix(&colour[1..=1], 16).ok().ok_or(format!("Could not parse character 1 of {colour} as a base 16 value"))?,
                0x10 * u8::from_str_radix(&colour[2..=2], 16).ok().ok_or(format!("Could not parse character 2 of {colour} as a base 16 value"))?,
                0x10 * u8::from_str_radix(&colour[3..=3], 16).ok().ok_or(format!("Could not parse character 3 of {colour} as a base 16 value"))?,
            ])
        } else if colour.len() == 6 {
            Ok([
                u8::from_str_radix(&colour[0..=1], 16).ok().ok_or(format!("Could not parse characters 0 and 1 of {colour} as a base 16 value"))?,
                u8::from_str_radix(&colour[2..=3], 16).ok().ok_or(format!("Could not parse characters 2 and 3 of {colour} as a base 16 value"))?,
                u8::from_str_radix(&colour[4..=5], 16).ok().ok_or(format!("Could not parse characters 4 and 5 of {colour} as a base 16 value"))?,
            ])
        } else if colour.len() == 7 {
            Ok([
                u8::from_str_radix(&colour[1..=2], 16).ok().ok_or(format!("Could not parse characters 1 and 2 of {colour} as a base 16 value"))?,
                u8::from_str_radix(&colour[3..=4], 16).ok().ok_or(format!("Could not parse characters 3 and 4 of {colour} as a base 16 value"))?,
                u8::from_str_radix(&colour[5..=6], 16).ok().ok_or(format!("Could not parse characters 5 and 6 of {colour} as a base 16 value"))?,
            ])
        } else {
            Err(format!("Colour value {colour} is not of the correct type"))
        }
    } else {
        Err(format!("Can't parse {colour} as a colour"))
    }
}

fn get_colours(input: &JsonValue) -> Result<Colours, String> {
    let colours = &input["colours"];
    if colours.is_null() {
        let mut cols: HashMap<i128, [u8; 3]> = HashMap::new();
        for col in DEFAULT_COLOURS {
            cols.insert(col.0, col.1);
        }
        return Ok(Colours { mapping: cols, default: DEFAULT_DEFAULT_COLOUR });
    };
    if !colours.is_object() {
        return Err("colours is not an object".into());
    };
    let mut default: [u8; 3] = DEFAULT_DEFAULT_COLOUR;
    let mut cols: HashMap<i128, [u8; 3]> = HashMap::new();
    for (value, col) in colours.entries() {
        if value == "default" {
            default = colour_to_u8_array(col)?;
        } else {
            match value.parse() {
                Ok(v) => {
                    cols.insert(v, colour_to_u8_array(col)?);
                },
                Err(_) => {
                    return Err(format!("Cannot parse {value} as a number"));
                }
            }
        }
    }

    Ok( Colours { mapping: cols, default } )
}



fn get_colour(value: i128, colours: &Colours) -> image::Rgb<u8> {
    image::Rgb(*colours.mapping.get(&value).unwrap_or(&colours.default))
}


fn execute_input(input: &JsonValue) -> Result<(), String> {
    let wall_type = get_wall_type(input)?;
    let sequence = get_sequence(input)?;
    let modulo = get_modulo(input)?;
    let left_values = get_left_values(input)?;
    let sequence_start = get_sequence_start(input)?;
    let colours = get_colours(input)?;

    let boundary = get_boundary(input)?;

    let output_file = get_output_file(input)?;
    
    match wall_type {
        WallType::Repeating => {
        match sequence {
        Sequence::Custom(s) => {
            if boundary.left < 0 {
                Err("Cannot have a left boundary less than 0".into())
            } else if boundary.right < 0 {
                Err("Cannot have a right boundary less than 0".into())
            } else {
                run_repeating_wall(s, modulo, boundary, output_file, colours);
                Ok(())
            }
        },
        _ => Err("Cannot use a pre defined sequence with the repeating wall type".into())
        }
        },
        WallType::LeftConst => {
            if boundary.left < 0 {
                Err("Cannot have a left boundary less than 0".into())
            } else if boundary.right < 0 {
                Err("Cannot have a right boundary less than 0".into())
            } else {
                match sequence {
                Sequence::Knight => {
                    run_left_const_fn_wall(left_const_functions::knight, left_values, modulo, boundary, output_file, colours);
                    Ok(())
                },
                Sequence::Rook => {
                    run_left_const_fn_wall(left_const_functions::rook, left_values, modulo, boundary, output_file, colours);
                    Ok(())
                },
                Sequence::Pagoda => {
                    run_left_const_fn_wall(left_const_functions::pagoda, left_values, modulo, boundary, output_file, colours);
                    Ok(())
                },
                Sequence::Rueppel => {
                    run_left_const_fn_wall(left_const_functions::rueppel, left_values, modulo, boundary, output_file, colours);
                    Ok(())
                },
                Sequence::Zigzag => {
                    run_left_const_fn_wall(left_const_functions::zigzag, left_values, modulo, boundary, output_file, colours);
                    Ok(())
                },
                Sequence::Custom(s) => {
                    run_left_const_wall(s, left_values, modulo, boundary, output_file, colours);
                    Ok(())
                },
                }
            }
        },
        WallType::BiDirectional => {
            match sequence {
                Sequence::Knight => {
                    run_bi_directional_fn_wall(bi_directional_functions::knight, modulo, boundary, output_file, colours);
                    Ok(())
                },
                Sequence::Rook => {
                    run_bi_directional_fn_wall(bi_directional_functions::rook, modulo, boundary, output_file, colours);
                    Ok(())
                },
                Sequence::Pagoda => {
                    run_bi_directional_fn_wall(bi_directional_functions::pagoda, modulo, boundary, output_file, colours);
                    Ok(())
                },
                Sequence::Rueppel => {
                    run_bi_directional_fn_wall(bi_directional_functions::rueppel, modulo, boundary, output_file, colours);
                    Ok(())
                },
                Sequence::Zigzag => {
                    run_bi_directional_fn_wall(bi_directional_functions::zigzag, modulo, boundary, output_file, colours);
                    Ok(())
                },
                Sequence::Custom(s) => {
                    run_bi_directional_wall(s, sequence_start, modulo, boundary, output_file, colours);
                    Ok(())
                },
            }
        }
    }
}

fn run_repeating_wall(sequence: Vec<i128>, modulo: i128, boundary: Boundary, output_file: String, colours: Colours) {
    let mut holder = repeating_sequence_wall::RepeatingSequenceWallHolder::new(sequence, modulo, boundary.top, boundary.bottom, boundary.left as usize, boundary.right as usize);

    let len:u32 = (boundary.right-boundary.left+1).try_into().unwrap();
    let height:u32 = (boundary.bottom-boundary.top).try_into().unwrap();

    let mut img = image::RgbImage::new(len, height+1);
    let mut y = 0;

    while let Some(_) = holder.calculate_next_line() {
        let line = holder.get_last_line().unwrap();
        
        for (x, val) in line.iter().enumerate() {
            let x = x.try_into().unwrap();
            img.put_pixel(x, y, get_colour(*val, &colours));
        }
        y += 1;
    }

    img.save(output_file).unwrap();
}

fn run_left_const_fn_wall<F>(sequence_func: F, left_values: [i128; 2], modulo: i128, boundary: Boundary, output_file: String, colours: Colours)
    where
        F: Fn(usize) -> i128
{
    let mut holder = left_const_wall::LeftConstWallHolder::new_from_sequence_func(sequence_func, left_values, modulo, boundary.top, boundary.bottom, boundary.left as usize, boundary.right as usize);

    let len:u32 = (boundary.right-boundary.left+1).try_into().unwrap();
    let height:u32 = (boundary.bottom-boundary.top).try_into().unwrap();

    let mut img = image::RgbImage::new(len, height+1);
    let mut y = 0;

    while let Some(_) = holder.calculate_next_line() {
        let line = holder.get_last_line().unwrap();
        
        for (x, val) in line.iter().enumerate() {
            let x = x.try_into().unwrap();
            img.put_pixel(x, y, get_colour(*val, &colours));
        }
        y += 1;
    }

    img.save(output_file).unwrap();
}

fn run_left_const_wall(sequence: Vec<i128>, left_values: [i128; 2], modulo: i128, boundary: Boundary, output_file: String, colours: Colours) {
    let mut holder = left_const_wall::LeftConstWallHolder::new(sequence, left_values, modulo, boundary.top, boundary.bottom, boundary.left as usize, boundary.right as usize);

    let len:u32 = (boundary.right-boundary.left+1).try_into().unwrap();
    let height:u32 = (boundary.bottom-boundary.top).try_into().unwrap();

    let mut img = image::RgbImage::new(len, height+1);
    let mut y = 0;

    while let Some(_) = holder.calculate_next_line() {
        let line = holder.get_last_line().unwrap();
        
        for (x, val) in line.iter().enumerate() {
            let x = x.try_into().unwrap();
            img.put_pixel(x, y, get_colour(*val, &colours));
        }
        y += 1;
    }

    img.save(output_file).unwrap();
}


fn run_bi_directional_fn_wall<F>(sequence_func: F, modulo: i128, boundary: Boundary, output_file: String, colours: Colours)
    where
        F: Fn(isize) -> i128
{
    let mut holder = bi_directional_wall::BiDirectionalWallHolder::new_from_sequence_func(sequence_func, modulo, boundary.top, boundary.bottom, boundary.left, boundary.right);

    let len:u32 = (boundary.right-boundary.left+1).try_into().unwrap();
    let height:u32 = (boundary.bottom-boundary.top).try_into().unwrap();

    let mut img = image::RgbImage::new(len, height+1);
    let mut y = 0;

    while let Some(_) = holder.calculate_next_line() {
        let line = holder.get_last_line().unwrap();
        
        for (x, val) in line.iter().enumerate() {
            let x = x.try_into().unwrap();
            img.put_pixel(x, y, get_colour(*val, &colours));
        }
        y += 1;
    }

    img.save(output_file).unwrap();
}

fn run_bi_directional_wall(sequence: Vec<i128>, sequence_start: isize, modulo: i128, boundary: Boundary, output_file: String, colours: Colours) {
    let mut holder = bi_directional_wall::BiDirectionalWallHolder::new(sequence, sequence_start, modulo, boundary.top, boundary.bottom, boundary.left, boundary.right);

    let len:u32 = (boundary.right-boundary.left+1).try_into().unwrap();
    let height:u32 = (boundary.bottom-boundary.top).try_into().unwrap();

    let mut img = image::RgbImage::new(len, height+1);
    let mut y = 0;

    while let Some(_) = holder.calculate_next_line() {
        let line = holder.get_last_line().unwrap();
        
        for (x, val) in line.iter().enumerate() {
            let x = x.try_into().unwrap();
            img.put_pixel(x, y, get_colour(*val, &colours));
        }
        y += 1;
    }

    img.save(output_file).unwrap();
}

