# 🎉 BytePiece
A [SentencePiece](https://github.com/google/sentencepiece) imitation. This is a state-of-the-art lossless compression algorithm that is a mix of n-grams and BPE (Byte pair encoding). You'll get the vocabulary model by training it with binary files, and compress it by encoding with the model. Training process: The unigram with the best subvector and scores will be saved in the training process, and the bytes will be sliced with BPE after that. Afterward, it compresses by encoding bytes into tokens based on statistics of the vocabulary model, and relying on byte patterns from suboptimal compression by other algorithms, for example, LZ77 (Lempel-Ziv 77), RLE (Run-length encoding), or Elias gamma coding. Reducing size: The compression ratio might be 5% at best for lossy files like JPG. But it also can find byte patterns to compress very well on vector graphics like SVG.  

# ⛓️ Dependencies for Windows/Linux
1.	[Get Scoop](https://scoop.sh/). Scoop will tell you to use PowerShell to install.  
2.	Get GCC with cmd `scoop install gcc` and close command prompt to reload the Windows PATH environment variables.  
3.	Run my custom build command in Python with [`compile.py`](compile.py).  

Try out the binary from [`output/bytepiece.exe`](output/bytepiece.exe)  
Skip step 1 and 2 if you're on Linux.  

# 🔧 Usage
/* Work in progress */

# 🧲 Why is this program truly useful❓
- ✅ Compressing lossy data: You want to understand the pattern of bytes in transform coded then entropy (DCT-II with Huffman coding) compressed files like [JPG](https://en.wikipedia.org/wiki/JPEG#JPEG_codec_example).  
/* Work in progress */

# 🗿 Why not this program❓
- ❎ Ignores the boundaries: This is not your LLM (Large language model) tokenizer, such as [tokenizers](https://github.com/huggingface/tokenizers) replacement. And, this tokenizes at byte level rather than subword level, so it may make training your LLM models much more difficult because it ignores the boundaries of the words or characters like space.  
/* Work in progress */

# ©️ License
All code in this repository belongs to no one: [Unlicense](UNLICENSE).  
[pexels-pixabay-302743.jpg](pexels-pixabay-302743.jpg) is placed under the same license [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/) without additional restrictions.  
I'll hold no liability for misuse of BytePiece. You are welcome to do anything legally with BytePiece.  

# 🤔 Debugging the program
1.	GDB debugger installation  
	-	`scoop install gdb`  
	-	To debug: `gdb --batch -ex run output/bytepiece`  
2.	[Cppcheck static analyzer](https://cppcheck.net/) installation
	-	[Download setup msi at Github](https://github.com/danmar/cppcheck/releases/) or `sudo apt-get install cppcheck` on Linux.  
	-	To debug: `Ctrl+Shift+O` to open a project file and select [bytepiece.cppcheck](bytepiece.cppcheck).  
3.	Clang static analyzer installation  
	-	`scoop install llvm`  
	-	To debug: `clang -Wall --analyze src/main.c`  
