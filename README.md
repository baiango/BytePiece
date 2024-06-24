# BytePiece
A [SentencePiece](https://github.com/google/sentencepiece) imitation. This is a lossless compression algorithm that is a mix of n-grams and byte pair encoding, it compresses based on statistics of the vocabulary model, and the missed repeating bytes by other algorithms. You'll get the vocabulary model by training it with binary files, and compress it by encoding with the model. The compression ratio might be 5% at best for lossy files like JPG.

# Dependencies
1.	[A Rust installation](https://www.rust-lang.org/learn/get-started)
2.	See the [Cargo.toml](Cargo.toml) for more.
```
[dependencies]
num_cpus = "1.16.0" # To see how many cores the system has
rayon = "1.10.0" # Use multiple threads
```

# Why is this program useful?
✅ Compressing compressed data: You want to understand the pattern of bytes in (DCT with Huffman coding compressed) files like [JPG](https://en.wikipedia.org/wiki/JPEG#JPEG_codec_example).  
✅ Portable and precise data types: All codes are written as a namespace, which is quite independent of local and external libraries, this means most source code files don't depend on each other. This program is precise in its data types due to Rust, that's why it's easily ported to another mid-level abstraction language like C and C++.  
✅ Complies in a few clicks or `cargo r -- --realese`: This program is written in Rust, so it won't leak and maximize memory uses.  
✅ Real-world automated testing: This algorithm comes with automated unit testing on a clear glass sphere JPG encoded image, and it's smart and stable, so it won't crash on your computer but only on the maintainer's computer, rest assured this is safe.  
✅ Runs on all cores: This program took 29 lines to run the trainer in all cores with Rayon.

# Why not this program
❎ Works on byte-level: This is not your LLM tokenizer such as [tokenizers](https://github.com/huggingface/tokenizers) replacement. It may make training your LLM models more difficult because it ignores the boundaries of the words or characters like space.  
❎ Memory size and bandwidth heavy but scalable: This algorithm depends on `BTree<Vec<u8> i16>` to store the keys in O(n^2) time. Training an 8 KiB chunk takes 16 GiB+ (n ** 2 * 256) of memory. But, this can scale down to 256 bytes of memory with a 1 byte chunk.  