# 🎉 BytePiece
A [SentencePiece](https://github.com/google/sentencepiece) imitation. This is a lossless compression algorithm that is a mix of n-grams and BPE (Byte pair encoding). The unigram with the best subvector and scores will be saved in the training process, and the bytes will be sliced with BPE after that. Afterward, it compresses by encoding bytes into tokens based on statistics of the vocabulary model, and relying on byte patterns from suboptimal compression by other algorithms, for example, LZ77 (Lempel-Ziv 77), RLE (Run-length encoding), or Elias gamma coding. You'll get the vocabulary model by training it with binary files, and compress it by encoding with the model. The compression ratio might be 5% at best for lossy files like JPG. But it also can find byte patterns to compress very well on vector graphics like SVG.  

# ⛓️ Dependencies
- TLDR: Install Rust and run `cargo r -- --release` in the project's directory.  
1.	[A Rust installation](https://www.rust-lang.org/learn/get-started)  
2.	See the [Cargo.toml](Cargo.toml) for more. It'll be installed by Cargo from Rust automatically.  
```
[dependencies]
num_cpus = "1.16.0" # To see how many cores the system has
rayon = "1.10.0" # Use multiple threads
```

# Why is this program useful❓
- ✅ Compressing lossy data: You want to understand the pattern of bytes in transform coded then entropy (DCT-II with Huffman coding) compressed files like [JPG](https://en.wikipedia.org/wiki/JPEG#JPEG_codec_example).  
- ✅ Portable and precise data types: All source files are treated as a namespace, which is quite independent of local and external libraries, this means most source code files don't depend on each other. This program is precise in its data types due to Rust, that's why it's easily ported to another mid-level abstraction language like C and C++.  
- ✅ Complies in a few clicks or `cargo r -- --release`: This program is written in Rust, so it won't leak and maximize memory uses.  
- ✅ Real-world automated testing: This algorithm comes with automated unit testing on a lightweight [clear glass sphere](pexels-pixabay-302743.jpg) JPG encoded image. It's sapient and dependable because of side-effectless functions, so it won't crash on your computer but only on the maintainer's. Rest assured this is safe.  
- ✅ Runs on all cores: This program took 29 lines to run the trainer in all cores with Rayon.

# Why not this program❓
- ❎ Ignores the boundaries: This is not your LLM (Large language model) tokenizer, such as [tokenizers](https://github.com/huggingface/tokenizers) replacement. And, this tokenizes at byte level rather than subword level, so it may make training your LLM models much more difficult because it ignores the boundaries of the words or characters like space.  
- ❎ Memory size and bandwidth heavy but scalable: This algorithm depends on `BTreeMap<Vec<u8>, i16>` to store the keys in O(n^2) time. Training an 8 KiB chunk takes 16 GiB+ (n ** 2 * 256) of memory. But, this can scale down to 256 bytes of memory with a 1 byte chunk. Without dropping out keys of memory optimization, this algorithm can only handle 3 KiB chunks on a 24 GiB of memory system.  
