use crate::{AABBQuery, AABBRelation, Enumerator};

struct BVHBranch<B> {
	bound: B,
	left: usize,
	right: usize,
}

struct BVHLeaf<B, V> {
	bound: B,
	value: V,
}

/// BVH is an immutable BVH which is designed to be as compact in
/// memory as possible.
///
/// It is recommended to bake BVH for static bodies as assets instead
/// of constructing them on the fly. Doing so will we be able to have
/// fine control over the granularities and depth of the finally
/// constructed BVH.
///
/// You may also distill constructed mutable BVH into this one.
///
/// The memory compactness of BVH is based on the fact that all leaf
/// nodes of the tree contains the value, and all branch nodes in the
/// tree contains references. So it won't be hard to prove there'll
/// always be n-1 branches for n nodes, and the mid-order traversal of
/// the hierarchy exhibits the node-branch-...-node (N-B-N) pattern:
///
///   1. For a single leaf node, the statement above is true.
///   2. For a branch node, it won't be hard to show:
///      i. There're n_left nodes with n_left-1 branches on the left
///         subtree, n_right nodes with n_right-1 branches on the right
///         subtree, then there're totally n_left+n_right nodes with
///         n_left+n_right-1 branches in current subtree.
///      ii. Traversal of the left subtree yields the N-B-N pattern,
///          then traversing the branch node yields the N-B-N-B pattern,
///          finally traversing right subtree yields N-B-N-B-N-B-N.
///
/// However, we'll always visit the branch nodes first, and then leaves,
/// so we place branches and leaves into different lists instead of
/// interleaving the nodes.
pub struct BVH<B, V> {
	root: usize,
	branches: Vec<BVHBranch<B>>,
	leaves: Vec<BVHLeaf<B, V>>,
}

// decompose bvhnode id attempt to decompose and check whether the id
// specifies a branch or node.
fn decompose(id: usize) -> (usize, bool) {
	(id >> 1, (id & 1) != 0)
}

impl<B, V> BVH<B, V> {
	// leftmost index of leaf for specified branch index.
	fn leftmost(&self, branch: usize) -> usize {
		let mut node = &self.branches[branch];
		loop {
			let (id, is_branch) = decompose(node.left);
			if !is_branch {
				return id;
			}
			node = &self.branches[id];
		}
	}

	// rightmost index of leaf for specified branch index.
	fn rightmost(&self, branch: usize) -> usize {
		let mut node = &self.branches[branch];
		loop {
			let (id, is_branch) = decompose(node.right);
			if !is_branch {
				return id;
			}
			node = &self.branches[id];
		}
	}

	/// query for all items hit by the AABB query and return.
	///
	/// Those items that are either included in or intersecting with
	/// the AABB query body will be returned. The enumerated order will
	/// be the same with the their specified order in the leaves' list,
	/// which has nothing to do with their depth.
	pub fn query<'a, 'b: 'a>(
		&'b self, q: &'a impl AABBQuery<B>,
	) -> impl 'a + Iterator<Item = &'b V> {
		Enumerator::new(|| {
			if self.root == 0 && self.leaves.len() == 0 {
				return;
			}

			// TODO(haoran.luo): actually the depth of tree is bounded,
			// we can also allocate the stack right at first.
			let mut stack: Vec<usize> = Vec::new();
			stack.push(self.root);
			while stack.len() > 0 {
				let top = stack[stack.len() - 1];
				let (id, is_branch) = decompose(top);
				let popstack;
				if is_branch {
					let branch = &self.branches[top];
					match q.check(&branch.bound) {
						AABBRelation::Interleave => popstack = true,
						AABBRelation::Intersect => {
							// XXX: we first put the left node onto the
							// stack, and later the right node will be
							// pushed when the left node is popped,
							// following the post-order traversal.
							stack.push(branch.left);
							popstack = false;
						},
						AABBRelation::Include => {
							// XXX: bingo 777! So lucky, we will just
							// yield all leaves under current subtree.
							let leftmost = self.leftmost(id);
							let rightmost = self.rightmost(id);
							for index in leftmost..=rightmost {
								yield &self.leaves[index].value;
							}
							popstack = true;
						},
					}
				} else {
					let leaf = &self.leaves[id];
					match q.check(&leaf.bound) {
						AABBRelation::Interleave => {},
						_ => yield &leaf.value,
					}
					popstack = true;
				}
				if popstack {
					stack.pop();
					let mut oldtop = top;
					while stack.len() > 0 {
						let newtop = stack[stack.len() - 1];
						let (index, is_branch) = decompose(newtop);
						assert!(is_branch);
						let parent = &self.branches[index];
						if oldtop == parent.left {
							stack.push(parent.right);
							break;
						} else {
							stack.pop();
							oldtop = newtop;
						}
					}
				}
			}
		})
	}
}
