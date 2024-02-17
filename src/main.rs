use regex::bytes::Regex;
use std::env;
use std::fs;
use std::io;
use std::io::BufRead;
use std::io::Read;
use std::io::Write;
use std::os::linux::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::process;
use std::time::SystemTime;

fn extract_args(args: &Vec<String>, start_index: usize, end_index: usize) -> Vec<String> {
	args[start_index..end_index].to_vec()
}

fn pwd() -> Result<(), io::Error> {
	let current_dir = env::current_dir()?;

	if let Some(path) = current_dir.to_str() {
		println!("{}", path);
	}

	Ok(())
}

fn echo(args: Vec<String>, option: char) {
	let mut first = true;

	for arg in args {
		if first == true {
			print!("{}", arg);
			first = false;
		} else {
			print!(" {}", arg);
		}
	}

	if option != 'n' {
		println!();
	}
}

fn cat(filename: &str) -> io::Result<()> {
	let mut file = fs::File::open(filename)?;

	let mut file_content = String::new();
	file.read_to_string(&mut file_content)?;

	print!("{}", file_content);
	Ok(())
}

fn mkdir(path: &str) -> io::Result<()> {
	fs::create_dir_all(path)?;

	Ok(())
}

fn rmdir(path: &str) -> io::Result<()> {
	fs::remove_dir(path)?;

	Ok(())
}

fn mv(src_path: &str, dest_path: &str) -> io::Result<()> {
	fs::rename(src_path, dest_path)?;

	Ok(())
}

fn ln(src_path: &str, dest_path: &str, option: char) -> io::Result<()> {
	if option == 's' {
		std::os::unix::fs::symlink(src_path, dest_path)?
	} else {
		fs::hard_link(src_path, dest_path)?
	}

	Ok(())
}

fn rm(path: &str, flag_r: bool, flag_d: bool) -> io::Result<()> {
	let metadata = fs::metadata(path)?;

	if metadata.is_file() || metadata.is_symlink() {
		if let Err(err) = fs::remove_file(path) {
			return Err(err);
		}
	} else if metadata.is_dir() {
		if flag_r == true {
			if let Err(err) = fs::remove_dir_all(path) {
				return Err(err);
			}
		} else if flag_d == true {
			if let Err(err) = fs::remove_dir(path) {
				return Err(err);
			}
		} else {
			// Attempt to remove directory without correct options
			return Err(io::Error::new(io::ErrorKind::Other, ""));
		}
	} else {
		// File or directory doesn't exist
		return Err(io::Error::new(io::ErrorKind::Other, ""));
	}

	Ok(())
}

fn ls_r(og_path: &str, path: &str, flag_a: bool) {
	let mut subdirs: Vec<String> = Vec::new();
	let path_p = Path::new(path);

	if path_p.is_dir() {
		println!("{}:", path);
		if flag_a {
			println!(".\n..");
		}

		if let Ok(entries) = fs::read_dir(path) {
			for entry in entries {
				if let Ok(entry) = entry {
					let entry_path = entry.path();
					if let Some(entry_path_str) = entry_path.to_str() {
						let mut og_p = og_path.to_string();
						og_p.push('/');
						if !flag_a {
							if !entry_path_str.contains("/.") {
								subdirs.push(entry_path_str.to_string());
								if let Some(rel_path) = entry_path.file_name() {
									println!("{}", rel_path.to_string_lossy());
								}
							}
						} else {
							subdirs.push(entry_path_str.to_string());
							if let Some(rel_path) = entry_path.file_name() {
								println!("{}", rel_path.to_string_lossy());
							}
						}
					}
				}
			}
		}
	}

	for subdir in subdirs {
		ls_r(og_path, &subdir, flag_a);
	}
}

fn ls(path: &str, flag_a: bool, flag_r: bool) -> io::Result<()> {
	let metadata = fs::metadata(path)?;

	if metadata.is_file() || metadata.is_symlink() {
		println!("{}", path);
	} else if metadata.is_dir() {
		if flag_a && !flag_r {
			println!(".\n..");
		}

		if flag_r {
			ls_r(path, path, flag_a);
		} else {
			let entries = fs::read_dir(path)?;
			for entry in entries {
				let entry = entry?;
				let entry_path = entry.path();
				if let Ok(rel_path) = entry_path.strip_prefix(path) {
					if let Some(rel_path_str) = rel_path.to_str() {
						if !flag_a {
							if !rel_path_str.starts_with(".") {
								println!("{}", rel_path_str);
							}
						} else {
							println!("{}", rel_path_str);
						}
					}
				} else {
					return Err(io::Error::new(io::ErrorKind::Other, ""));
				}
			}
		}
	}

	Ok(())
}

fn copy(src_path: &str, dest_path: &str) -> io::Result<()> {
	let mut src_file = fs::File::open(src_path)?;
	let dest_path = Path::new(dest_path);
	let mut dest_file = if dest_path.is_dir() {
		if let Some(src_file_name_osstr) = Path::new(src_path).file_name() {
			if let Some(src_file_name) = src_file_name_osstr.to_str() {
				let dest_file_path = dest_path.join(src_file_name);
				fs::File::create(dest_file_path)?
			} else {
				return Err(io::Error::new(io::ErrorKind::Other, ""));
			}
		} else {
			return Err(io::Error::new(io::ErrorKind::Other, ""));
		}
	} else {
		fs::File::create(dest_path)?
	};

	let mut buffer = [0; 4096];

	loop {
		let bytes = src_file.read(&mut buffer)?;
		if bytes == 0 {
			break;
		}
		dest_file.write_all(&buffer[0..bytes])?;
	}

	Ok(())
}

fn get_cp_dir_name(src_path: &str, dest_path: &str) -> PathBuf {
	if !Path::new(dest_path).exists() {
		Path::new(dest_path).to_path_buf()
	} else {
		if let Some(source_dir_name_osstr) = Path::new(src_path).file_name() {
			if let Some(source_dir_name) = source_dir_name_osstr.to_str() {
				Path::new(dest_path).join(source_dir_name)
			} else {
				PathBuf::new()
			}
		} else {
			PathBuf::new()
		}
	}
}

fn copy_r(src_path: &str, dest_path: &str) -> io::Result<()> {
	if !Path::new(src_path).is_dir() {
		copy(src_path, dest_path)?;
	} else {
		let dest_dir_path = get_cp_dir_name(src_path, dest_path);
		fs::create_dir_all(&dest_dir_path)?;

		for entry in fs::read_dir(src_path)? {
			let entry = entry?;
			let entry_path = entry.path();
			if let Some(entry_file_name) = entry_path.file_name() {
				if let Some(entry_name) = entry_file_name.to_str() {
					if let Some(dest_dir_name) = dest_dir_path.to_str() {
						let dest_entry_path = Path::new(dest_dir_name).join(entry_name);
						if entry_path.is_dir() {
							copy_r(
								&entry_path.to_string_lossy(),
								&dest_entry_path.to_string_lossy(),
							)?;
						} else {
							copy(
								&entry_path.to_string_lossy(),
								&dest_entry_path.to_string_lossy(),
							)?;
						}
					}
				}
			}
		}
	}

	Ok(())
}

fn cp(flag_r: bool, src_path: &str, dest_path: &str) -> io::Result<()> {
	if !flag_r {
		if let Err(_) = copy(src_path, dest_path) {
			return Err(io::Error::new(io::ErrorKind::Other, ""));
		}
	} else {
		if let Err(_) = copy_r(src_path, dest_path) {
			return Err(io::Error::new(io::ErrorKind::Other, ""));
		}
	}

	Ok(())
}

fn touch(path: &str, flag_a: bool, flag_c: bool, flag_m: bool) -> io::Result<()> {
	let path = Path::new(path);

	if path.exists() || flag_c {
		if flag_a || flag_m {
			let mut file = fs::File::open(&path)?;
			let mut file_contents = String::new();
			file.read_to_string(&mut file_contents)?;

			let _ = fs::remove_file(&path);
			let mut modified_file = fs::File::create(&path)?;
			modified_file.write_all(file_contents.as_bytes())?;
		}
	} else {
		let _ = fs::File::create(&path)?;
	}

	Ok(())
}

fn grep(file: &str, pattern: &str) -> io::Result<()> {
	let file = fs::File::open(file)?;
	let read = io::BufReader::new(file);

	if let Ok(regex) = Regex::new(pattern) {
		for (_, line) in read.lines().enumerate() {
			let line = line?;
			if regex.is_match(&line.as_bytes()) {
				println!("{}", line);
			}
		}
	}

	Ok(())
}

fn format_permissions(mode: u32) -> String {
	let user = format!(
		"{}{}{}",
		if mode & 0o400 != 0 { 'r' } else { '-' },
		if mode & 0o200 != 0 { 'w' } else { '-' },
		if mode & 0o100 != 0 { 'x' } else { '-' }
	);
	let group = format!(
		"{}{}{}",
		if mode & 0o40 != 0 { 'r' } else { '-' },
		if mode & 0o20 != 0 { 'w' } else { '-' },
		if mode & 0o10 != 0 { 'x' } else { '-' }
	);
	let others = format!(
		"{}{}{}",
		if mode & 0o4 != 0 { 'r' } else { '-' },
		if mode & 0o2 != 0 { 'w' } else { '-' },
		if mode & 0o1 != 0 { 'x' } else { '-' }
	);

	format!("{}{}{}", user, group, others)
}

fn get_user_name(uid: u32) -> String {
	let passwd_path = Path::new("/etc/passwd");
	if let Ok(file) = fs::File::open(&passwd_path) {
		let reader = io::BufReader::new(file);
		for line in reader.lines() {
			if let Ok(line) = line {
				let parts: Vec<&str> = line.split(':').collect();
				if parts.len() >= 3 {
					if let Ok(parsed_uid) = parts[2].parse::<u32>() {
						if parsed_uid == uid {
							return parts[0].to_string();
						}
					}
				}
			}
		}
	}
	uid.to_string()
}

fn get_group_name(gid: u32) -> String {
	let group_path = Path::new("/etc/group");
	if let Ok(file) = fs::File::open(&group_path) {
		let reader = io::BufReader::new(file);
		for line in reader.lines() {
			if let Ok(line) = line {
				let parts: Vec<&str> = line.split(':').collect();
				if parts.len() >= 3 {
					if let Ok(parsed_gid) = parts[2].parse::<u32>() {
						if parsed_gid == gid {
							return parts[0].to_string();
						}
					}
				}
			}
		}
	}
	gid.to_string()
}

fn format_modified_time(modified_time: SystemTime) -> String {
	let datetime: chrono::DateTime<chrono::Local> = modified_time.into();
	datetime.format("%-e %H:%M").to_string()
}

fn ls_l(path: &str) -> Result<(), std::io::Error> {
	let mut entries: Vec<_> = fs::read_dir(path)?.filter_map(Result::ok).collect();

	// https://stackoverflow.com/questions/40021882/how-to-sort-readdir-iterator
	entries.sort_by_key(|entry| entry.file_name().to_ascii_lowercase());

	for entry in entries {
		let path = entry.path();
		let metadata = fs::metadata(&path)?;

		let file_name = entry.file_name();
		let file_name_str = file_name.to_str().unwrap();
		let file_type = if metadata.is_dir() { "d" } else { "-" };

		let permissions = metadata.permissions().mode();
		let mode = format_permissions(permissions);
		let size = metadata.len();
		let modified = metadata.modified()?;

		let owner = metadata.st_uid();
		let group = metadata.st_gid();

		let owner_name = get_user_name(owner);
		let group_name = get_group_name(group);

		if !file_name_str.starts_with(".") {
			println!(
				"{}{} {} {} {} {} {}",
				file_type,
				mode,
				owner_name,
				group_name,
				size,
				format_modified_time(modified),
				file_name_str
			);
		}
	}

	Ok(())
}

fn check_valid_params(given_param: &str, valid_params: Vec<&str>) -> bool {
	let mut is_valid = false;

	for param in valid_params {
		if param == given_param {
			is_valid = true;
			break;
		}
	}

	is_valid
}

fn chmod(perm: &str, file: &str) -> io::Result<()> {
	if let Some(first_char) = perm.chars().next() {
		if first_char.is_alphabetic() {
			// True for plus, false for minus/negative (pun intended)
			let mut operator = true;
			let identity = if perm.contains('+') {
				perm.split("+").next()
			} else {
				operator = false;
				perm.split("-").next()
			};

			if let Some(identity) = identity {
				let mut permission: &str = "";
				if operator {
					if let Some(separator_index) = perm.find('+') {
						permission = &perm[separator_index + 1..perm.len()];
					}
				} else {
					if let Some(separator_index) = perm.find('-') {
						permission = &perm[separator_index + 1..perm.len()];
					}
				}

				let metadata = fs::metadata(file)?;
				let mut curr_perm = metadata.permissions().mode();

				if operator {
					for p in permission.chars() {
						if p == 'r' {
							for i in identity.chars() {
								match i {
									'u' => curr_perm |= 0o400,
									'g' => curr_perm |= 0o040,
									'o' => curr_perm |= 0o004,
									'a' => curr_perm |= 0o444,
									_ => {}
								}
							}
						} else if p == 'w' {
							for i in identity.chars() {
								match i {
									'u' => curr_perm |= 0o200,
									'g' => curr_perm |= 0o020,
									'o' => curr_perm |= 0o002,
									'a' => curr_perm |= 0o222,
									_ => {}
								}
							}
						} else if p == 'x' {
							for i in identity.chars() {
								match i {
									'u' => curr_perm |= 0o100,
									'g' => curr_perm |= 0o010,
									'o' => curr_perm |= 0o001,
									'a' => curr_perm |= 0o111,
									_ => {}
								}
							}
						}
					}
				} else {
					for p in permission.chars() {
						if p == 'r' {
							for i in identity.chars() {
								match i {
									'u' => curr_perm &= !0o400,
									'g' => curr_perm &= !0o040,
									'o' => curr_perm &= !0o004,
									'a' => curr_perm &= !0o444,
									_ => {}
								}
							}
						} else if p == 'w' {
							for i in identity.chars() {
								match i {
									'u' => curr_perm &= !0o200,
									'g' => curr_perm &= !0o020,
									'o' => curr_perm &= !0o002,
									'a' => curr_perm &= !0o222,
									_ => {}
								}
							}
						} else if p == 'x' {
							for i in identity.chars() {
								match i {
									'u' => curr_perm &= !0o100,
									'g' => curr_perm &= !0o010,
									'o' => curr_perm &= !0o001,
									'a' => curr_perm &= !0o111,
									_ => {}
								}
							}
						}
					}
				}

				let new_perm = fs::Permissions::from_mode(curr_perm);
				fs::set_permissions(file, new_perm)?;
			}
		} else {
			if let Ok(octal_num) = u32::from_str_radix(perm, 8) {
				let new_perm = fs::Permissions::from_mode(octal_num);
				fs::set_permissions(file, new_perm)?;
			}
		}
	}

	Ok(())
}

fn main() -> Result<(), i32> {
	let args: Vec<String> = env::args().collect();

	for i in 1..args.len() {
		match args[1].as_str() {
			"pwd" => {
				let _ = pwd();
				return Ok(());
			}
			"echo" => {
				if i + 1 == args.len() {
					println!();
				} else if args[i + 1] == "-n" {
					let echo_args = extract_args(&args, i + 2, args.len());
					echo(echo_args, 'n');
				} else {
					let echo_args = extract_args(&args, i + 1, args.len());
					echo(echo_args, ' ');
				}
				return Ok(());
			}
			"cat" => {
				let mut error_active = false;
				let cat_files = extract_args(&args, i + 1, args.len());
				for file in cat_files {
					if let Err(_) = cat(&file) {
						error_active = true;
						break;
					}
				}

				if error_active {
					process::exit(236);
				}

				return Ok(());
			}
			"mkdir" => {
				let mut error_active = false;
				let mkdir_paths = extract_args(&args, i + 1, args.len());
				for path in mkdir_paths {
					if let Err(_) = mkdir(&path) {
						error_active = true;
						break;
					}
				}

				if error_active {
					process::exit(226);
				}

				return Ok(());
			}
			"rmdir" => {
				let mut error_active = false;
				let rmdir_paths = extract_args(&args, i + 1, args.len());
				for path in rmdir_paths {
					if let Err(_) = rmdir(&path) {
						error_active = true;
						break;
					}
				}

				if error_active {
					process::exit(196);
				}

				return Ok(());
			}
			"mv" => {
				let src_path = &args[i + 1];
				let dest_path = &args[i + 2];
				if let Err(_) = mv(&src_path, &dest_path) {
					process::exit(216);
				}

				return Ok(());
			}
			"ln" => {
				if args[i + 1] == "-s" || args[i + 1] == "--symbolic" {
					let src_path = &args[i + 2];
					let dest_path = &args[i + 3];
					if let Err(_) = ln(&src_path, &dest_path, 's') {
						process::exit(206);
					}
				} else {
					let valid_params = vec!["-s", "--symbolic"];
					if args[i + 1].starts_with("-")
						&& !check_valid_params(&args[i + 1], valid_params)
					{
						println!("Invalid command");
						process::exit(255);
					} else {
						let src_path = &args[i + 1];
						let dest_path = &args[i + 2];
						if let Err(_) = ln(&src_path, &dest_path, 'h') {
							process::exit(206);
						}
					}
				}
				return Ok(());
			}
			"rm" => {
				let mut flag_r = false;
				let mut flag_d = false;
				let mut error_active = false;

				for j in i + 1..args.len() {
					if args[j] == "-r" || args[j] == "-R" || args[j] == "--recursive" {
						flag_r = true;
						if args.len() < 4 {
							println!("Invalid command");
							process::exit(255);
						}
					} else if args[j] == "-d" || args[j] == "-D" || args[j] == "--dir" {
						flag_d = true;
						if args.len() < 4 {
							println!("Invalid command");
							process::exit(255);
						}
					} else {
						let rm_dir_paths = extract_args(&args, j, args.len());
						for path in rm_dir_paths {
							if let Err(_) = rm(&path, flag_r, flag_d) {
								error_active = true;
							}
						}
						break;
					}
				}

				if error_active == true {
					process::exit(186);
				}

				return Ok(());
			}
			"ls" => {
				let mut flag_a = false;
				let mut flag_r = false;
				let mut error_active = false;

				if i + 1 == args.len() {
					let ls_path = ".";
					if let Err(_) = ls(&ls_path, flag_a, flag_r) {
						error_active = true;
					}
				} else {
					for j in i + 1..args.len() {
						if args[j] == "-R" {
							flag_r = true;
						} else if args[j] == "-a" {
							flag_a = true;
						} else if args[j] == "-l" {
							let ls_path = if args.len() == j + 1 {
								"."
							} else {
								&args[j + 1]
							};
							if let Err(_) = ls_l(&ls_path) {
								error_active = true;
							}
							break;
						} else {
							let ls_path = &args[j];
							if let Err(_) = ls(&ls_path, flag_a, flag_r) {
								error_active = true;
							}
							break;
						}
					}
				}

				if error_active {
					process::exit(176);
				}

				return Ok(());
			}
			"cp" => {
				let mut flag_r = false;
				let mut error_active = false;

				if args[i + 1] == "-r" || args[i + 1] == "-R" || args[i + 1] == "--recursive" {
					flag_r = true;
					if let Err(_) = cp(flag_r, &args[i + 2], &args[i + 3]) {
						error_active = true;
					}
				} else if i + 2 == args.len() - 1 {
					if let Err(_) = cp(flag_r, &args[i + 1], &args[i + 2]) {
						error_active = true;
					}
				} else {
					error_active = true;
				}

				if error_active {
					process::exit(166);
				}

				return Ok(());
			}
			"touch" => {
				let mut flag_a = false;
				let mut flag_m = false;
				let mut flag_c = false;
				let mut error_active = false;

				for j in i + 1..args.len() {
					if args[j] == "-a" {
						flag_a = true;
					} else if args[j] == "-m" {
						flag_m = true;
					} else if args[j] == "-c" || args[j] == "--no-create" {
						flag_c = true;
					} else {
						if let Err(_) = touch(&args[j], flag_a, flag_c, flag_m) {
							error_active = true;
						}
					}
				}

				if error_active {
					process::exit(156);
				}

				return Ok(());
			}
			"chmod" => {
				let perm = &args[i + 1];

				if perm.len() < 3 {
					println!("Invalid command");
					process::exit(255);
				}

				let file = &args[i + 2];
				if let Err(_) = chmod(&perm, &file) {
					process::exit(231);
				}

				return Ok(());
			}
			"grep" => {
				let pattern = &args[i + 1];
				let path = &args[i + 2];
				if let Err(_) = grep(&path, &pattern) {
					process::exit(255);
				}

				return Ok(());
			}
			_ => {
				println!("Invalid command");
				process::exit(255);
			}
		}
	}

	Ok(())
}
