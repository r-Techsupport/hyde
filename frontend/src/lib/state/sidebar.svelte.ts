import type { INode } from '$lib/types';
import { apiAddress } from '$lib/main';

/**
 * The filesystem tree for the document folder
 */
export const documentTree: INode = $state({
	name: '',
	children: []
});

export const assetTree: INode = $state({
	name: '',
	children: []
});

export async function loadSidebarInfo() {
	// Fetch the document tree
	const docResponse = await fetch(`${apiAddress}/api/tree/doc`);
	const reportedDocTree = await docResponse.json();
	documentTree.name = reportedDocTree.name;
	documentTree.children = reportedDocTree.children;

	// Fetch the asset tree
	const assetResponse = await fetch(`${apiAddress}/api/tree/asset`);
	const reportedAssetTree = await assetResponse.json();
	assetTree.name = reportedAssetTree.name;
	assetTree.children = reportedAssetTree.children;
}
