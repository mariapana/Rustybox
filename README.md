[![Review Assignment Due Date](https://classroom.github.com/assets/deadline-readme-button-24ddc0f5d75046c5622901739e7c5dd533143b0c8e959d652212380cedb1ea36.svg)](https://classroom.github.com/a/iYoQzOhX)
# Rustybox

Rustybox is a busybox written entirely in Rust. It implements Linux bash command utilities depending on the given input.

#### Available commands:
* **pwd**
* **echo** [-n] arguments
* **cat** files
* **mkdir** dir_name
* **mv** source destination
* **ln** [-s] source link_name
* **rm** [-r/-R/--recursive/-d/--dir] files/directories
* **ls** [-a/--all/-R/--recursive/-l] [directory]
* **cp** [-r/-R/--recursive] source destination
* **touch** [-a/-m/-c] file
* **chmod** permissions file
* **grep** regex file

## Function implementations and descriptions

```rust
extract_args(args: &Vec<String>, start_index: usize, end_index: usize) -> Vec<String>
```
* Extracts a range of arguments from a given vector of strings.
* It takes the vector of arguments, a starting index, and an ending index as input and returns a new vector containing the extracted arguments.

```rust
pwd() -> Result<(), io::Error>
```
* Prints the current working directory to the standard output.
* It uses the env::current_dir() function to get the current directory and then prints it if successful.

```rust
echo(args: Vec<String>, option: char)
```
* Prints the provided list of arguments to the standard output, separated by spaces.
* The option parameter, if 'n', omits the trailing newline character.
* Formats the printed output to avoid trailing whitespace.


```rust
cat(filename: &str) -> io::Result<()>
```
* Reads and prints the contents of a file specified by the filename parameter.
* It opens the file, reads its contents, and prints them to the standard output.
* If it encounters an error, it exits the program with an error code.


```rust 
mkdir(path: &str) -> io::Result<()>
```
* Attempts to create a directory specified by the path parameter.
* If successful, it creates the directory and returns Ok(()).
* If it encounters an error, it exits the program with an error code.

```rust
rmdir(path: &str) -> io::Result<()>
```
* Attempts to remove a directory specified by the path parameter.
* If successful, it creates the directory and returns Ok(()).
* If it encounters an error, it exits the program with an error code.

```rust
mv(src_path: &str, dest_path: &str) -> io::Result<()>
```
* Attempts to rename or move a file or directory from src_path to dest_path.
* It uses the fs::rename function to perform the operation.
* If successful, it creates the directory and returns Ok(()).
* If it encounters an error, it exits the program with an error code.

```rust
ln(src_path: &str, dest_path: &str, option: char) -> io::Result<()>
```
* Creates a hard link or a symbolic link based on the provided option.
* It uses the fs::hard_link or std::os::unix::fs::symlink function to create the link.
* If successful, it creates the directory and returns Ok(()).
* If it encounters an error, it exits the program with an error code.

```rust
rm(path: &str, flag_r: bool, flag_d: bool) -> io::Result<()>
```
* Attempts to remove a file or directory specified by the path parameter.
* The flag_r and flag_d parameters determine whether to perform a recursive deletion or a directory deletion.
* If successful, it creates the directory and returns Ok(()).
* If it encounters an error, it exits the program with an error code.

```rust
ls(path: &str, flag_a: bool, flag_r: bool) -> io::Result<()>
```
* Lists the contents of a file or directory specified by the path parameter.
* The flag_a parameter determines whether hidden files and directories are included.
* The flag_r parameter enables recursive listing by calling ls_r().
* It checks if the given path exists and is a directory using fs::metadata. If it's a directory and flag_r is true, it calls the ls_r function to perform a recursive listing. If it's a directory and flag_r is false, it lists the contents of the directory.
* It reads the entries in the directory using fs::read_dir(path). For each entry, it checks if it should be displayed (based on the flag_a parameter) and prints the entry's name to the standard output. If the path is a regular file, it simply prints the filename.

```rust
ls_r(og_path: &str, path: &str, flag_a: bool)
```
* Recursively lists the contents of a directory specified by the path parameter.
* It can include hidden files and directories if the flag_a parameter is set to true.
* It uses fs::read_dir(path) to read the contents of the directory. For each entry in the directory, it checks if it should be displayed (based on the flag_a parameter) and prints the entry's name to the standard output.
* If an entry is a subdirectory, it recursively calls itself with the new subdirectory as the path. The function maintains and prints the directory structure by calculating the relative path based on the original path (og_path).

```rust
ls_l(path: &str) -> Result<(), std::io::Error>
```
* Lists the contents of a directory in a detailed, "long" format.
* It displays information about file types, permissions, owner, group, size, modification time, and filename (gathered from file metadata, /etc/passwd or /etc/group).

```rust
get_user_name(uid: u32) -> String
```
* ls_l() helper function that looks up and extracts the username associated with a given user ID (UID) on the system by reading /etc/passwd.

```rust
get_group_name(gid: u32) -> String
```
* ls_l() helper function that looks up and extracts the group name associated with a given group ID (GID) on the system by reading /etc/group.

```rust
format_modified_time(modified_time: SystemTime) -> String
```
* ls_l() helper function that formats a SystemTime object as a string representing the last modified time of a file in the format "day hour:minute".

```rust
cp(flag_r: bool, src_path: &str, dest_path: &str) -> io::Result<()>
```
* Copies files and directories. 
* It uses the copy function for regular files and the copy_r function for directories.
* The flag_r parameter controls whether the copy operation is recursive.

```rust
copy(src_path: &str, dest_path: &str) -> io::Result<()>
```
* cp() helper function that creates or opens the destination file using File::create(dest_path) to write the content.
* If the destination is a directory, it appends the source file's name to the destination directory path.
* It then reads the content of the source file and writes it to the destination file using io::copy.

```rust
get_cp_dir_name(src_path: &str, dest_path: &str) -> PathBuf
```
* Used to determine the destination directory name when copying a file or directory.
* It checks if the destination exists and appends the source filename accordingly. 
* It constructs a Path from the dest_path. It appends the base name of src_path (the source file or directory name) to the destination path. 
* It returns the resulting PathBuf, which represents the destination directory or file path, taking into account the source file or directory name.

```rust
copy_r(src_path: &str, dest_path: &str) -> io::Result<()>
```
* cp() helper function that recursively copying a source directory and its contents to a destination directory.
* It uses fs::read_dir(src_path) to read the contents of the source directory. For each entry in the source directory, it checks if it's a file or a subdirectory.
* If it's a file, it calls the copy function to copy the file from the source to the destination directory.
* If it's a directory, it recursively calls itself with the subdirectory as the new source and constructs a corresponding destination path within the destination directory.

```rust
touch(path: &str, flag_a: bool, flag_c: bool, flag_m: bool)
```
* Creates or updates a file specified by the path, based on the given parameters.
* It checks if the file specified by path exists using fs::metadata. 
* If the file exists, and both flag_a and flag_m are false, the function does nothing by returning Ok(()). This is because there is no need to update the access or modification times if both flags are false.
* If the file doesn't exist and flag_c is true, the function does nothing by returning Ok(()). This ensures that a new file is not created when flag_c is set, and the file doesn't exist.
* If the file doesn't exist or flag_c is true, it creates the file using fs::File::create(&path).
* If flag_a or flag_m is true, it reads the current content of the file using fs::File::open(&path) and then immediately writes the same content back to the file. This has the effect of updating the access or modification time of the file without changing its content.

```rust
grep(file: &str, pattern: &str) -> io::Result<()>
```
* Searches for lines in a file specified by the file parameter that match a given regular expression pattern. It prints the matching lines to the standard output.
* It creates a buffered reader (io::BufReader) to efficiently read the contents of the file line by line.
* It checks if the provided pattern is a valid regular expression using the Regex::new function from the regex crate. If the pattern is not a valid regular expression, the function will not proceed with the search.
* The function then iterates through the lines of the file. For each line, it checks if the line matches the provided regular expression.
* If a line matches the pattern, it prints the matching line to the standard output.
* The function continues to read the next line and repeats the matching process until it reaches the end of the file.

```rust
chmod(perm: &str, file: &str) -> io::Result<()>
```
* Changes the permissions of a file specified by the file parameter according to the permission string provided in perm. It supports both symbolic and octal representations of permissions.
* The function checks the first character of the perm string to determine whether it is symbolic (starts with an alphabet character) or octal (starts with a numeric character).
If symbolic notation is used, it checks whether the + or - operator is present in the perm string to identify whether the operation should be additive (add permissions) or subtractive (remove permissions).
* If symbolic notation is used, the function extracts the identity and permission parts from the perm string. It then updates the permission bits according to the symbolic notation. It then applies the requested permission changes by modifying the current permission bits based on the provided identity and permissions.
* If the perm string starts with a numeric character, it's interpreted as an octal permission mode.
The octal number is converted to an integer and the resulting value represents the new permission mode.

```rust
main() -> Result<(), i32>
```
* Parses command-line arguments, dispatches commands to their respective functions, and handles errors or invalid commands by exiting the program with the appropriate exit code.

## Verify

Run the following commands to test your homework:

You will have to install NodeJS (it is installed in the codespace)

```bash
# Clone tests repository
git submodule update --init 

# Update tests repository to the lastest version
cd tests
git pull 
cd ..

# Install loadash
npm install lodash
```

Install rustybox

```bash
cargo install --path .
```

If the `rustybox` command can't be found, be sure to add the default cargo installation folder into the PATH environment variable

```bash
export PATH=/home/<your username here>/.cargo/bin:$PATH
```

Run tests

```bash
cd tests
# Run all tests 
./run_all.sh

# Run single test
./run_all.sh pwd/pwd.sh
```
