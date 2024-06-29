#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include "stle.h"
#include "tok_trainer.h"


ErrorCode tok_trainer_train_unigram_bytes(TrieNode *root, Bytes *bytes) {
	for (mut_u32 i = 0; i < bytes->len - 1; ++i) {
		printf("%x,", bytes->data[i]);
	}
	printf("%x\n", bytes->data[bytes->len - 1]);

	for (mut_u32 i = 0; i < bytes->len; ++i) {
		for (mut_u32 j = i; j < bytes->len; ++j) {
			Bytes slice = {
				.data = (mut_u8 *)malloc(sizeof(mut_u8) * (j - i + 1)),
				.len = j - i + 1
			};
			memcpy(slice.data, &bytes->data[i], slice.len * sizeof(u8));

			stle_trie_insert(root, &slice);
			TrieNode *child_node = root;
			stle_trie_get(&child_node, &slice);
			child_node->value += 1;
			free(slice.data);
		}
	}
	return OK;
}

ErrorCode test_tok_trainer_train_unigram_bytes() {
	Bytes bytes_arr = {
		.data = (mut_u8[]){
			1, 2, 3, 1, 2, 3, 1, 2,
			3, 1, 2, 3, 1, 2, 3, 1,
		},
		.len = 16
	};
	TrieNode *test_trie = stle_trie_new();
	tok_trainer_train_unigram_bytes(test_trie, &bytes_arr);

	stle_trie_prt_all(test_trie);
	return OK;
}

ErrorCode test_tok_trainer_modules() {
	ErrorCode result = OK;
	result += test_tok_trainer_train_unigram_bytes();
	return result;
}
