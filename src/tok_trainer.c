#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include "stle.h"
#include "tok_trainer.h"


ErrorCode tok_trainer_train_unigram_bytes_a(TrieNode *root, Bytes *bytes) {
	for (mut_u32 i = 0; i < bytes->len; ++i) {
		for (mut_u32 j = i; j < bytes->len; ++j) {
			Bytes slice = {
				.data = (mut_u8 *)malloc(sizeof(mut_u8) * (j - i + 1)),
				.len = j - i + 1
			};
			// Memory write and read reduction idea: Compress slices with a JPG tokenizer
			// Save computations idea: Skip if the slice is smaller the storing byte size
			memcpy(slice.data, &bytes->data[i], slice.len * sizeof(u8));

			// There's a bug where it stops inserting at 3rd index in this loop
			stle_trie_insert_unchecked(root, &slice);
			TrieNode *child_node = root;
			stle_trie_get_unchecked(&child_node, &slice);
			child_node->value += 1;
			free(slice.data);
		}
	}
	return OK;
}

ErrorCode tok_trainer_get_best_vec_value(subvec_and_val *ret, TrieNode *root) {
	u8 byte_size = 1;


	return OK;
}

ErrorCode tok_trainer_test_train_unigram_bytes_a() {
	Bytes bytes_arr = {
		.data = (mut_u8[]){
			1, 2, 3, 1, 2, 3, 1, 2,
			3, 1, 2, 3, 1, 2, 3, 1,
		},
		.len = 16
	};
	TrieNode *test_trie = stle_trie_new();
	tok_trainer_train_unigram_bytes_a(test_trie, &bytes_arr);

	stle_trie_prt_all_unchecked(test_trie);
	return OK;
}

ErrorCode tok_trainer_test_modules() {
	ErrorCode result = OK;
	result += tok_trainer_test_train_unigram_bytes_a();
	return result;
}
