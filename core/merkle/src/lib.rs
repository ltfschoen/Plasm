#![cfg_attr(not(feature = "std"), no_std)]

use rstd::prelude::*;
use sr_primitives::traits::{Member, MaybeSerializeDebug, Hash};
use support::storage::child;
use parity_codec::{Encode, Codec};

pub mod mock;

pub fn concat_hash<H, F>(a: &H, b: &H, hash: F) -> H
	where H: Encode + Default + Eq + Copy,
		  F: FnOnce(&[u8]) -> H {
	if *a == Default::default() { return *b; }
	if *b == Default::default() { return *a; }
	hash(&plasm_primitives::concat_bytes(a, b))
}

// H: Hash, O: Outpoint(Hashable)
pub trait MerkleTreeTrait<H, Hashing>
	where H: Codec + Member + MaybeSerializeDebug + rstd::hash::Hash + AsRef<[u8]> + AsMut<[u8]> + Copy + Default,
		  Hashing: Hash<Output=H>
{
	/// get root Hash of MerkleTree.
	fn root() -> H;
	/// get proofs of leaf.
	fn proofs(leaf: &H) -> MerkleProof<H>;
	/// push Hash to MerkleTree.
	fn push(leaf: H);
	// commit to MerkleTree
	fn commit();
}

pub trait MerkleDb<Id: Encode, Key: Encode, O: Codec> {
	fn push(&self, trie_id: &Id, key: &Key, o: O) {
		child::put_raw(&trie_id.encode()[..], &key.encode()[..], &o.encode()[..]);
	}
	fn get(&self, trie_id: &Id, key: &Key) -> Option<O> {
		if let Some(ret) = child::get_raw(&trie_id.encode()[..], &key.encode()[..]) {
			return O::decode(&mut &ret[..]);
		}
		return None;
	}
}

pub struct DirectMerkleDb;

impl<Id: Encode, Key: Encode, O: Codec> MerkleDb<Id, Key, O> for DirectMerkleDb {}

pub trait ProofTrait<H>
	where H: Codec + Member + MaybeSerializeDebug + rstd::hash::Hash + AsRef<[u8]> + AsMut<[u8]> + Copy + Default {
	fn root<Hashing>(&self) -> H where Hashing: Hash<Output=H>;
	fn leaf(&self) -> &H;
	fn proofs(&self) -> &Vec<H>;
	fn depth(&self) -> u8;
	fn index(&self) -> u64;
}

#[derive(Debug)]
pub struct MerkleProof<H> {
	pub proofs: Vec<H>,
	pub depth: u8,
	pub index: u64,
}

impl<H> MerkleProof<H>
	where H: Codec + Member + MaybeSerializeDebug + rstd::hash::Hash + AsRef<[u8]> + AsMut<[u8]> + Copy + Default
{
	fn re_root<Hashing>(&self, mid: u64, now_l: usize, now_r: usize) -> H
		where Hashing: Hash<Output=H>
	{
		if now_r - now_l == 1 {
			return concat_hash(&self.proofs[now_l], &self.proofs[now_r], Hashing::hash);
		}
		let now_depth = self.proofs.len() as u64 - now_r as u64 + now_l as u64;
		let new_mid = (1u64 << self.depth >> now_depth) + mid;
		if new_mid <= self.index {
			return concat_hash(&self.proofs[now_l],
							   &self.re_root::<Hashing>(new_mid, now_l + 1, now_r),
							   Hashing::hash);
		} else {
			return concat_hash(&self.re_root::<Hashing>(mid, now_l, now_r - 1),
							   &self.proofs[now_r],
							   Hashing::hash);
		}
	}

	fn re_leaf(&self, mid: u64, now_l: usize, now_r: usize) -> &H {
		if now_r == now_l {
			return &self.proofs[now_r];
		}
		let now_depth = self.proofs.len() as u64 - now_r as u64 + now_l as u64;
		let new_mid = (1u64 << self.depth >> now_depth) + mid;
		if new_mid <= self.index {
			return &self.re_leaf(new_mid, now_l + 1, now_r);
		} else {
			return &self.re_leaf(mid, now_l, now_r - 1);
		}
	}
}

impl<H> ProofTrait<H> for MerkleProof<H>
	where H: Codec + Member + MaybeSerializeDebug + rstd::hash::Hash + AsRef<[u8]> + AsMut<[u8]> + Copy + Default {
	fn root<Hashing>(&self) -> H
		where Hashing: Hash<Output=H>
	{
		self.re_root::<Hashing>(0, 0, self.proofs.len() - 1)
	}

	fn leaf(&self) -> &H { self.re_leaf(0, 0, self.proofs.len() - 1) }

	fn proofs(&self) -> &Vec<H> { &self.proofs }
	fn depth(&self) -> u8 { self.depth }
	fn index(&self) -> u64 { self.index }
}

#[cfg(test)]
mod tests;
