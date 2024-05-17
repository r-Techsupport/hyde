<script lang="ts">
    import DOMPurify from "dompurify";
    import { onMount } from "svelte";

    enum INodeType {
        File,
        Directory
    }

    // interface INodeEntry {
    //     name: string,
    //     nodeType: INodeType,
    //     // children: T extends INodeType.Directory ? INode<INodeType>[] : null;
    // }

    // interface INodeDirectory extends INodeEntry{
    //     children: INodeEntry[] | INodeDirectory[];
    // }

    interface INode<T extends INodeType> {
        name: string,
        nodeType: INodeType,
        children: T extends INodeType.Directory ? INode<INodeType>[] : null;
    }

    const MOCK_DIRECTORY: INode<INodeType.Directory> = {
    // const MOCK_DIRECTORY: INodeDirectory = {
        name: "Root",
        nodeType: INodeType.Directory,
        children: [
            {
                name: "File 1",
                nodeType: INodeType.File,
                children: null,
            },
            {
                name: "File 2",
                nodeType: INodeType.File,
                children: null,
            },
            {
                name: "Directory 1",
                nodeType: INodeType.Directory,
                children: [
                    {
                        name: "File1 in dir 1",
                        nodeType: INodeType.File,
                        children: null,
                    }
                ]
            }
        ],
    }

    let directoryNav: HTMLElement;

    /**
     * Render the provided node into an html string
     * 
     * I hope god forgives me for what I have done
     * @param iNode
     */
    function renderINode(iNode: INode<INodeType>): string {
        if (iNode.nodeType === INodeType.File) {
            return `<li class="i-node">${iNode.name}</li>`;
        } else {
            let innerContent = "";
            // non null assertion: If it's a directory, it must have children
            for (const node of iNode.children!) {
                innerContent += renderINode(node);
            }
            const id = `${iNode.name.replaceAll(" ", "-")}-contents`;
            // TODO: figure out how to exempt this code from getting caught by dompurify
            // return DOMPurify.sanitize(
                return `<li onclick="\
                                    const e = document.getElementById('${id}');\
                                    if (e.style.display === 'none') {e.style.display = '';} \
                                    else {e.style.display = 'none';} \
                                    " class="i-node">${iNode.name}</li>
                <ul class="i-node" id="${id}">${innerContent}</ul>`
            // );
        }
    }

    $: onMount(() => {
        directoryNav.innerHTML += renderINode(MOCK_DIRECTORY)
    });
</script>

<div class="side-bar">
    <ul bind:this={directoryNav} class="directory-nav">
    </ul>
</div>

<style>
	@import '/css/theme.css';
	.side-bar {
		background-color: var(--background-1);
		width: var(--sidebar-width);
		height: 100vh;
        color: var(--foreground-0);
		font-family:
			system-ui,
			-apple-system,
			BlinkMacSystemFont,
			'Segoe UI',
			Roboto,
			Oxygen,
			Ubuntu,
			Cantarell,
			'Open Sans',
			'Helvetica Neue',
			sans-serif;
	}

    .side-bar :global(.i-node) {
        /* padding-left: 1rem; */
        padding-left: 0.5rem;
        border-left: 1px solid var(--foreground-5);
        margin-left: 0;
        list-style-type: none;
        padding-bottom: 0.1rem;
        padding-top: 0.1rem;
        font-size: large;
        cursor: pointer;
    }

    .directory-nav {
        margin-top: 10rem;
        padding-left: 0;
    }

    /* .side-bar > * {
        list-style-type: none;
    }

    .side-bar ul {
        list-style-type: none;
    } */

</style>
