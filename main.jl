install = false
if install
	import Pkg
	Pkg.add("CUDA")
	Pkg.add("FileIO")
end
# using CUDA
import Base: ==
using FileIO


const View{T} = SubArray{T, 1, Vector{T}, Tuple{UnitRange{Int64}}, true}

mutable struct Range
	start::Int64
	finish::Int64
end

function ==(a::Range, b::Range)::Bool
	return a.start == b.start && a.finish == b.finish
end

mutable struct Trie
	children::Vector{Union{Trie, Nothing}}
	value::UInt32
end

function trie_new()::Trie
	return Trie(Vector{Union{Trie, Nothing}}(nothing, typemax(UInt8) + 1), 0)
end

function trie_get_ref_unchk!(root::Trie, key::Union{Vector{UInt8}, SubArray{UInt8}})::Trie
	for byte in key
		if root.children[byte] === nothing
			root.children[byte] = trie_new()
		end
		root = root.children[byte]
	end
	return root
end

function trie_prt_unchk(root::Trie)::Nothing
	function prt_recursive(root::Trie, prefix::Vector{UInt8}, depth::UInt16)::Nothing
		if root.value != 0
			println(prefix, "    ", root.value)
		end

		for (byte, child) in enumerate(root.children)
			if child !== nothing
				prt_recursive(child, push!(copy(prefix), UInt8(byte)), UInt16(depth + 1))
			end
		end
	end
	prt_recursive(root, Vector{UInt8}(), UInt16(0));
end

function train_unigram_unchk!(root::Trie, bytes_views::Vector{View{UInt8}}, skip_small_bytes::UInt16=UInt16(1))::Nothing
	for view_ in bytes_views
		for start in skip_small_bytes:length(view_)
			for finish in start + 1:length(view_)
				slice = view(view_, start:finish)
				trie_get_ref_unchk!(root, slice).value += 1
			end
		end
	end
end

function get_best_overhead_unchk(root::Trie, byte_size::UInt8=UInt8(1))::Tuple{Vector{UInt8}, UInt32}
	best_prefix = UInt8[]
	best_value::UInt32 = 0

	function score_recursive(root::Trie, prefix::Vector{UInt8}, depth::UInt16)::Nothing
		score::UInt32 = max(0, root.value * depth - root.value * byte_size)

		if best_value < score
			best_prefix = prefix
			best_value = score
		end

		for (byte, child) in enumerate(root.children)
			if child !== nothing
				score_recursive(child, push!(copy(prefix), UInt8(byte)), UInt16(depth + 1))
			end
		end
	end
	score_recursive(root, UInt8[], UInt16(0))
	return best_prefix, best_value
end

function search_all_patterns(bytes::View{UInt8}, pattern::Vector{UInt8})::Vector{UInt32}
	matches::Vector{UInt32} = []
	pattern_length::UInt32 = length(pattern)
	for i in 1:(length(bytes) - pattern_length + 1)
		if bytes[i] == pattern[1] # Mini Rabin-Karp w/o hash
			if bytes[i:(i + pattern_length - 1)] == pattern # Brute-force
				push!(matches, i)
			end
		end
	end
	return matches
end

function complement_range(input::Range, exclude::Range)::Vector{Range}
	not_overlapping::Bool = input.start > exclude.finish || input.finish < exclude.start
	if not_overlapping
		return [Range(input.start, input.finish)]
	end

	result::Vector{Range} = []
	if input.start < exclude.start
		append!(result, [Range(input.start, exclude.start)])
	end
	if input.finish > exclude.finish
		append!(result, [Range(exclude.finish, input.finish)])
	end
	return result
end

function cutoff_views(bytes::Vector{UInt8}, views::Vector{View{UInt8}}, pattern::Vector{UInt8})::Vector{View{UInt8}}
	new_view::Vector{View{UInt8}} = []

	for view_ in views
		for position in search_all_patterns(view_, pattern)
			bytes_range = Range(view_.indices[1].start, view_.indices[1].stop)
			exclude_range = Range(position, position + length(pattern))
			ranges::Vector{Range} = complement_range(bytes_range, exclude_range)
			for range in ranges
				push!(new_view, view(bytes, range.start:range.finish))
			end
		end
	end
	return new_view
end

function subtract_one!(model)::Nothing
	for (i, vec) in enumerate(model)
		vec[1][:] .-= 1
		model[i] = vec
	end
end

function train_tokenizer(bytes::Vector{UInt8})::Vector{Tuple{Vector{UInt8}, UInt32}}
	slices::Vector{View{UInt8}} = [view(bytes, 1:length(bytes))]
	model::Vector{Tuple{Vector{UInt8}, UInt32}} = []
	for _ in 1:5
		node = trie_new()
		train_unigram_unchk!(node, slices)
		best_subbyte::Tuple{Vector{UInt8}, UInt32} = get_best_overhead_unchk(node)

		println(best_subbyte)
		if (UInt8[], 0x00000000) == best_subbyte
			break
		end

		push!(model, best_subbyte)
		slices = cutoff_views(bytes, slices, model[1][1])
	end
	return model
end

function test_trie()::Nothing
	root::Trie = trie_new()
	hello_bytes::Vector{UInt8} = collect(codeunits("hello"))
	child = trie_get_ref_unchk!(root, hello_bytes)
	child.value = 8
	# trie_prt_unchk(root)
	@assert 8 == child.value
end

function test_complement_range()::Nothing
	result = complement_range(Range(1, 10), Range(3, 7))
	@assert [Range(1, 3), Range(7, 10)] == result

	result = complement_range(Range(10, 20), Range(3, 7))
	@assert [Range(10, 20)] == result

	result = complement_range(Range(1, 3), Range(3, 7))
	@assert [Range(1, 3)] == result

	result = complement_range(Range(7, 10), Range(3, 7))
	@assert [Range(7, 10)] == result
end

function test_search_all()::Nothing
	f = open("pexels-pixabay-302743.jpg", "r")
	data = read(f)
	close(f)
	pattern = UInt8[204, 204, 238, 204, 204, 238, 204, 204, 238]

	slice = view(data, 1:length(data))
	answer = UInt32[0x569e, 0x56a1, 0x56a4, 0x572e, 0x5731, 0x5734, 0x5737, 0xbce5]
	@assert answer == search_all_patterns(slice, pattern)
end

function test_train_tokenizer()::Nothing
	bytes = UInt8[
		1, 2, 3, 1, 2, 3, 1, 2,
		3, 1, 2, 3, 1, 2, 3, 1,
	]
	model = train_tokenizer(bytes)
	println(model)
	@assert model == [(UInt8[1, 2, 3, 1, 2, 3, 1, 2, 3, 1], 27), (UInt8[1, 2, 3, 1], 12)]
end

function tests()::Nothing
	test_trie()
	test_complement_range()
	test_search_all()
	test_train_tokenizer()
end

@time tests()
@time test_train_tokenizer()

const MAX_BYTES = UInt32(0xfffff)
buffer = zeros(UInt8, MAX_BYTES)

f = open("pexels-pixabay-302743.jpg", "r")
seekstart(f)
data = read!(f, buffer)
close(f)

model = train_tokenizer(data)
println(model)
