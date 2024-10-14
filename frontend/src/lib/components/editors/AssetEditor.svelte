<script lang="ts">
	import type { INode } from "$lib/main";
    import { assetTree } from "$lib/main";

    export let assetFolderPath = '';
    let tree: INode = {
        name: 'loading',
        children: []
    };

    assetTree.subscribe(t => tree = t);
    if (assetFolderPath !== '') {
        let currentDir = tree.children.find(v => v.name === assetFolderPath);
        console.log(currentDir?.children);
    }
</script>

<div class="status-bar">
    <!-- TODO: better message for no directory selected -->
    <p>You're viewing the assets for <strong>"{assetFolderPath}"</strong></p>
</div>

<style>
    :root {
        font-family: var(--font-family);
        color: var(--foreground-2);
    }

    .status-bar {
        border: none;
        border-bottom: 0.05rem solid var(--foreground-4);
    }

    .status-bar p {
        padding-left: 0.3rem;
    }
</style>
