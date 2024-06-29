#pragma once
#include "stle.h"


// typedef struct Range {
// 	u32 *bounds[2];
// } Range;

typedef struct subvec_and_val {
	u8 *subvector;
	i16 score;
} subvec_and_val;

ErrorCode tok_trainer_train_unigram_bytes(TrieNode *root, Bytes *bytes);
ErrorCode tok_trainer_test_train_unigram_bytes();
ErrorCode tok_trainer_test_modules();
