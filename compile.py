import subprocess
import os
import glob


c_file = glob.glob("src/*.c")
program_name = "bytepiece"
output_folder = f"./output/{program_name}"
optimized_program = False

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
# subprocess.run(["gcc", "-Wall", "-std=c17", *c_file, "-S", *flags, f"-o{output_folder}.asm"], capture_output=True, text=True)

gcc_result = subprocess.run(["gcc", "-Wall", "-std=c17", *c_file, *flags, f"-o{output_folder}.exe"], capture_output=True, text=True)
print("gcc stderr: --------------------------------------------------------------------\n", gcc_result.stderr)

clang_result = subprocess.run(["clang", "-Wall", "-std=c17", "--analyze", *c_file], capture_output=True, text=True)
print("clang stderr: ------------------------------------------------------------------\n", clang_result.stderr)


if gcc_result.returncode != 0:
	print("Error compiling C code:", gcc_result.stderr)
else:
	subprocess.run(["gdb", "--batch", "-ex", "run", output_folder + ".exe"])
