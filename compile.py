import subprocess
import os

c_file = "main.c"
program_name = "bytepiece"
output_folder = f"./output/{program_name}"

flags = [
		"-mavx2",
		"-O2",
		"-ftree-vectorize",
		"-fopt-info-vec",
		"-fopt-info-missed",
		"-Wall",
]

if not os.path.exists('output'):
	os.makedirs('output')

result = subprocess.run(["gcc", c_file, *flags, f"-o{output_folder}.exe"], capture_output=True, text=True)
print("stderr:", result.stderr)

subprocess.run(["gcc", c_file, "-S", *flags, f"-o{output_folder}.asm",], capture_output=True, text=True)

if result.returncode != 0:
	print("Error compiling C code:", result.stderr)
else:
	# If the compilation was successful, execute the compiled program
	subprocess.run([output_folder + ".exe"])
