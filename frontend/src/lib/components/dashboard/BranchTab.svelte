<script lang="ts">
    import { allBranches, me } from '$lib/main'; // Import your store with all branches and user data
    import { Permission } from '$lib/types';

    // Check if the user has the ManageBranches permission
    const canAccess = $me.permissions?.includes(Permission.ManageBranches);

    const sortedBranches = $allBranches.slice().sort((a, b) => a.name.localeCompare(b.name));
</script>

{#if canAccess}
  <div class="container">
    <ul>
      {#each sortedBranches as branch}
        <li>
          <label class="checkbox-label">
            <input type="checkbox" bind:checked={branch.isProtected} />
            {branch.name}
          </label>
        </li>
      {/each}
    </ul>
  </div>
{:else}
  <p>You do not have permission to view this page.</p>
{/if}

<style>
    .container {
        width: 100%;
        display: flex;
        justify-content: flex-start;
    }

    ul {
        list-style-type: none;
        padding: 0;
        margin: 0;
        margin-left: 1rem;
        width: 100%;
    }

    .checkbox-label {
        display: flex;
        align-items: center;
        width: 100%;
        height: 3rem;
        cursor: pointer;
        box-sizing: border-box;
        transition: background-color 0.3s ease;
    }

    .checkbox-label input {
        margin-right: 10px;
    }

    .checkbox-label:hover {
        background-color: var(--background-3);
    }

    .checkbox-label input:disabled {
        cursor: not-allowed;
        opacity: 0.5;
    }

    p {
        color: var(--foreground-2);
        text-align: center;
        margin-top: 2rem;
        font-size: 1.2rem;
    }
</style>
