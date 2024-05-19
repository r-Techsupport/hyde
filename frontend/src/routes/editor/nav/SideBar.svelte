<script lang="ts">
    import FileNavigation from "./FileNavigation.svelte";
    import DOMPurify from "dompurify";
    import { onMount } from "svelte";

    const MOCK_DIRECTORY = {
        name: "Root",
        children: [
            {
                name: "File 1",
                children: [],
            },
            {
                name: "File 2",
                children: [],
            },
            {
                name: "Directory 1",
                children: [
                    {
                        name: "File1 in dir 1",
                        children: [],
                    }
                ]
            },
            {
                name: "Directory 2",
                children: [
                    {
                        name: "file 1 in dir 2",
                        children: [],

                    }
                ]
            }
        ],
    }
</script>

<div class="side-bar">
    <!-- Because FileNavigation renders recursively, -->
    <!-- any "outside" css needs to be done in a separate div -->
    <div class="directory-nav">
        <FileNavigation {...MOCK_DIRECTORY}/>
    </div>
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
        margin-top: 2rem;
    }
</style>
