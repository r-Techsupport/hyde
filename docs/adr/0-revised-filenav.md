# ADR 0, Revised File Navigation
<https://github.com/adr/madr/blob/4.0.0/template/adr-template.md>

Author: James Boehme (zleyyij/arc)

Date: Aug 31, 2026

# Context and Problem Statement
Hyde currently makes a distinction between *assets*, and *documents*.

In the frontend, "documents" and "assets" are split into two separate
sections. When looking at documents, you can only look at documents. To 
modify assets, a separate viewer and filenav tree exist.

In the repository, Hyde enforces the assumption that there are separate
folders for assets and documents. When fetching a document, it asserts a
UTF-8 content type and assumes it's an Astro markdown document. When
fetching an asset, it asserts that it's an image or video.

The backend is selective about exposing the directory structure through
the `/api/tree/doc` (`get_doc_tree_handler`) and
`/api/tree/asset`(`get_asset_tree_handler`) endpoints.

This assumption makes it easy to handle content types, but impacts the
rest of the codebase. Parts of the codebase, like the state management
that stores the currently open file in the frontend don't make a
clear distinction between whether you're currently looking at an *asset*,
or a *document*. This has proven to be brittle, issues regularly occur
with faulty file resolution, and this assumption requires massive parts
of the codebase to acknowledge things they shouldn't strictly need to
care about. It makes it difficult to showcase the currently edited file
in the URL.

# Considered Option
The proposed change involves removing any filesystem distinction between
assets, and documents.

## Frontend Changes
In the frontend, the asset tree and the document tree will be combined
into a single tree. The file navigation will include a small icon to
differentiate between documents and assets (possibly also folders?),
displayed next to the file.

The correct editor will be opened *when* a file is selected, rather than
making the currently open editor a piece of global state that the user
unknowingly decides by clicking on one of two root folders.

The URL of the currently open file will be set in the navigation bar,
i.e `hyde.com/edit/docs/file2.md`. 

## Backend Changes
In the backend, `/api/files` would return the file tree, and `/api/files/foo/bar.md` would the file referenced, with a correctly set HTTP
`Content-Type` header. `/api/tree/doc` and `/api/tree/asset` would be removed.

The endpoints `/api/doc` and `/api/asset` are to be removed.