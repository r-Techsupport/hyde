/**
 * @file
 * An LRU cache
 */
import { apiAddress } from './main';

/**
 * The the type of the value stored in the cache
 */
type CacheEntry = string;

class AssetCache {
	// This will probably need to be updated to add support for non-text
	// stuff
	private values: Map<string, CacheEntry> = new Map<string, CacheEntry>();
	private maxEntries: number = 10;

	/**
	 * Fetch an asset, fetching it from the the cache first, or the network if it exists.
	 * @param path The path of the asset to fetch
	 */
	public async get(path: string): Promise<CacheEntry | null> {
		const hasKey = this.values.has(path);
		let entry: CacheEntry;
		// Re-insert to mark it as most recently accessed
		if (hasKey) {
			// non-null assertion: checked above with hasKey
			entry = this.values.get(path)!;
			this.values.delete(path);
			this.values.set(path, entry);
			return entry;
		}

		// Try to fetch it from the API if the value isn't found in memory
		const response = await fetch(`${apiAddress}/api/doc?path=${encodeURIComponent(path)}`);
		if (response.status === 200) {
			const value = (await response.json()).contents;
			this.set(path, value);
			return value;
		}

		// at this point, a value was either returned, or nothing was found
		return null;
	}

	/**
	 * Add a new item to the cache, evicting old items if necessary
	 * @param path The key to store the entry under
	 * @param value The entry
	 */
	set(path: string, value: CacheEntry) {
		// evict the least recently used item if necessary
		if (this.values.size >= this.maxEntries) {
			const keyToDelete = this.values.keys().next().value;
			// non-null assertion: We know there's at least one key left because of the above size check
			this.values.delete(keyToDelete!);
		}

		this.values.set(path, value);
	}

	/**
	 * Remove the specified item from cache.
	 *
	 * This operation is **not write-through**, so changes
	 * are not synced to the backend.
	 * @param path
	 */
	delete(path: string) {
		this.values.delete(path);
	}

	/**
	 * Completely empty the cache of all entries.
	 */
	flush() {
		this.values.clear();
	}
}

/**
 * A transparent caching layer that stores the last few documents accessed locally.
 */
export const cache: AssetCache = new AssetCache();
