/**
 * @file
 * This file contains code used for rendering markdown code
 */
import { marked } from "marked";

/**
 * Compile the provided input string into markdown and render it,
 * editing the provided html element
 * @param input The raw markdown to be rendered
 * @param output The element to insert the markdown into
 */
export async function renderMarkdown(input: string, output: InnerHTML): Promise<undefined> {
    // while this might seem redundant now, there's some pipeline stuff that'll
    // need to happen later
    // https://marked.js.org/#demo
    output.innerHTML = await marked.parse(input);

}