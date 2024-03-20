// Parses 2 files with the format "{dexnumber},{name}" on each line into their respective enums
//  and functions

use std::*;
use std::iter::Iterator;
use std::fs::*;
use std::io::prelude::*;

fn main() {
	let mut real_names = fs::read_to_string("realnames.txt")
        .unwrap_or_else( |_| { panic!("`realnames.txt` does not exist in directory."); String::new() } )
		.lines()
		.map(String::from)
		.collect::<Vec<String>>();
	let mut enum_names = fs::read_to_string("enumnames.txt")
        .unwrap_or_else( |_| { panic!("`enumnames.txt` does not exist in directory."); String::new() } )
		.lines()
		.map(String::from)
		.collect::<Vec<String>>();

	// HP,Attack,Defense,Sp.Attack,Sp.Defense,Speed
    let mut battle_types = fs::read_to_string("types.txt")
        .unwrap_or_else( |_| { panic!("`types.txt` does not exist in directory."); String::new() } )
		.lines()
		.map(String::from)
		.collect::<Vec<String>>();
	let mut base_stats = fs::read_to_string("stats.txt")
        .unwrap_or_else( |_| { panic!("`stats.txt` does not exist in directory."); String::new() } )
		.lines()
		.map(String::from)
		.collect::<Vec<String>>();

	let mut pimpl = fs::File::create("mons_impl.txt").unwrap();
	let mut penum = fs::File::create("mons_enum.txt").unwrap();
	
	let mut dex: Vec<i32> = Vec::new();
	let mut rname_vec = Vec::new();
	let mut ename_vec = Vec::new();
	let mut stats_vec: Vec<String> = Vec::new();
	let mut types_vec = Vec::new();

	let mut curvec = Vec::new();
	let mut curnamevec = Vec::new();
	for i in 0..real_names.len() {
		let rline = real_names[i].clone().chars().collect::<Vec<char>>();
		let eline_chars = enum_names[i].clone().chars().collect::<Vec<char>>();
		let mut e_idx = 0;
		curvec = Vec::new();
		curnamevec = Vec::new();
		for (i, rchar) in rline.clone().iter().enumerate() {
			let echar = match i > eline_chars.len()-1 {
				true => ' ',
				false => eline_chars[e_idx],
			};
			
			if echar as u8 >= b'0' && echar as u8 <= b'9' && i < 6 {
				curvec.push(echar);
			} else if echar == ',' {
				
				dex.push( curvec.clone().into_iter()
							.skip_while(|&ch| ch == '0')
							.collect::<String>()
							.parse()
							.unwrap());
				curvec = Vec::new();
			} else {
				curvec.push(echar.clone());
				curnamevec.push(rchar.clone());
			}

			e_idx += 1;
		}

		ename_vec.push(curvec.clone().into_iter().collect::<String>() );
		rname_vec.push(curnamevec.clone().into_iter().collect::<String>() );
	}

	// populate types
	for i in 0..battle_types.len() {
		types_vec.push( battle_types[i].split(',')
			.map(String::from)
			.collect::<Vec<String>>()
			.iter()
			.map(|word|
				if word.len() > 1 {
					format!("BattleType::{}",word).clone()
				} else {
					"BattleType::None".to_string()
				})
			.collect::<Vec<String>>() );
	}

	let mut code = String::new();

	code.push_str("#![allow(snake_case)]\n");
	code.push_str("#![allow(non_camel_case_types)]\n");
	code.push_str("#![allow(unused_variables)]\n\n");

	code.push_str("\n\nuse crate::types::BattleType;\n");
	
	code.push_str("\n\n#[derive(Debug, Clone, Copy)]\n");
	code.push_str("pub enum Pokemon {\n");	
	for word in ename_vec.iter() {
		code.push_str(format!("\t{},\n", word).as_str());
	}
	code.push_str("}\n");

	penum.write(code.as_bytes());
	
	code = String::new();
	
	// impl block
	code.push_str("impl Pokemon {\n");

	// get dex number
	code.push_str("\tpub fn dex(&self) -> u16 {\n");
	code.push_str("\t\tmatch self {\n");
	for i in 0..dex.len() {
		code.push_str(format!("\t\t\tSelf::{} => {}u16,\n", ename_vec[i], dex[i]).as_str());
	}
	code.push_str("\t\t\t_ => { println!(\"Unknown Pokémon ID. \"); u16::MAX },\n");
	code.push_str("\t\t}\n");
	code.push_str("\t}\n\n");
	
	// get name string
	code.push_str("\t\tpub fn name(&self) -> String {\n");
	code.push_str("\t\tmatch self {\n");
	for i in 0..dex.len() {
		code.push_str(format!("\t\t\tSelf::{} => \"{}\".to_string(),\n", ename_vec[i], rname_vec[i]).as_str());
	}
	code.push_str("\t\t\t_ => { println!(\"Unknown Pokémon ID. \"); String::new() },\n");
	code.push_str("\t\t}\n");
	code.push_str("\t}\n\n");

	// get base stats
	code.push_str("\t\tpub fn base_stats(&self) -> [u8; 6] {\n");
	code.push_str("\t\tmatch self {\n");
	for i in 0..dex.len() {
		code.push_str(format!("\t\t\tSelf::{} => [{}],\n", ename_vec[i], base_stats[i]).as_str());
	}
	code.push_str("\t\t\t_ => { println!(\"Unknown Pokémon ID. \"); [0u8,0u8,0u8,0u8,0u8,0u8] },\n");
	code.push_str("\t\t}\n");
	code.push_str("\t}\n\n");

	// get type(s) of pokemon
	code.push_str("\t\tpub fn types(&self) -> [BattleType; 2] {\n");
	code.push_str("\t\tmatch self {\n");
	for i in 0..dex.len() {
		code.push_str(format!("\t\t\tSelf::{} => [{}, {}],\n", ename_vec[i], types_vec[i][0], types_vec[i][1]).as_str());
	}
	code.push_str("\t\t\t_ => { println!(\"Unknown Pokémon ID. \"); [BattleType::None, BattleType::None] },\n");
	code.push_str("\t\t}\n");
	code.push_str("\t}\n\n");


		
	// close impl
	code.push_str("}\n\n");

	// get Pokemon from dex number
	code.push_str("pub fn mon_at_dex(dex: usize) -> Pokemon {\n");
	code.push_str("\tmatch dex as u16 {\n");
	for i in 0..dex.len() {
		code.push_str(format!("\t\t{} => Pokemon::{},\n", dex[i], ename_vec[i]).as_str());
	}
	code.push_str("\t\t_ => { println!(\"Invalid dex number. \"); Pokemon::Bulbasaur },\n");
	code.push_str("\t}\n}\n\n");
	
	pimpl.write(code.as_bytes());
	println!("End.");
	
}
