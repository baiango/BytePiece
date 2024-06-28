import subprocess
import os

c_file = "src/main.c"
program_name = "bytepiece"
output_folder = f"./output/{program_name}"
optimized_program = True

flags = [
	"-Wall",
	"-g",
	"-ftree-vectorize",
	"-fopt-info-vec",
	# "-fopt-info-missed", # Too verbose
]

if optimized_program:
	flags += [
		"-mavx2",
		"-O2",
	]

if not os.path.exists('output'):
	os.makedirs('output')

# Too large to read the ASM
# subprocess.run(["gcc", c_file, "-S", *flags, f"-o{output_folder}.asm"], capture_output=True, text=True)

gcc_result = subprocess.run(["gcc", c_file, *flags, f"-o{output_folder}.exe"], capture_output=True, text=True)
print("gcc stderr:", gcc_result.stderr)

clang_result = subprocess.run(["clang", "-Wall", "--analyze", c_file], capture_output=True, text=True)
print("clang stderr:", clang_result.stderr)


if gcc_result.returncode != 0:
	print("Error compiling C code:", gcc_result.stderr)
else:
	subprocess.run(["gdb", "--batch", "-ex", "run", output_folder + ".exe"])