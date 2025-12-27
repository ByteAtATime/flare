import { PaginatedExtensionsResponseSchema, type Extension } from '$lib/store';
import { fetch } from '@tauri-apps/plugin-http';

export type SortOption = 'default' | 'downloads' | 'recent' | 'oldest';

export class ExtensionsStore {
	extensions = $state<Extension[]>([]);
	searchResults = $state<Extension[]>([]);
	featuredExtensions = $state<Extension[]>([]);
	trendingExtensions = $state<Extension[]>([]);

	isLoading = $state(true);
	isSearching = $state(false);
	error = $state<string | null>(null);

	#_searchText = $state('');
	selectedCategory = $state('All Categories');
	selectedIndex = $state(0);
	sortBy = $state<SortOption>('default');

	currentPage = $state(1);
	isFetchingMore = $state(false);
	hasMore = $state(true);

	readonly perPage = 50;
	#searchDebounceTimer: ReturnType<typeof setTimeout> | undefined;
	#loadingDebounceTimer: ReturnType<typeof setTimeout> | undefined;

	allCategories = $derived.by(() => {
		const categories = new Set<string>();
		const allFetched = [...this.featuredExtensions, ...this.trendingExtensions, ...this.extensions];
		for (const ext of allFetched) {
			if (ext.categories) {
				for (const cat of ext.categories) {
					categories.add(cat);
				}
			}
		}
		return ['All Categories', ...Array.from(categories).sort()];
	});

	constructor() {
		this.#fetchInitialData();
	}

	get searchText() {
		return this.#_searchText;
	}

	set searchText(value: string) {
		this.#_searchText = value;
		clearTimeout(this.#searchDebounceTimer);
		clearTimeout(this.#loadingDebounceTimer);

		if (!value) {
			this.searchResults = [];
			this.isLoading = false;
			this.isSearching = false;
			if (this.error) this.error = null;
			return;
		}

		this.isSearching = true;

		this.#loadingDebounceTimer = setTimeout(() => {
			this.isLoading = true;
		}, 200);

		this.#searchDebounceTimer = setTimeout(async () => {
			if (value !== this.#_searchText) {
				return;
			}
			this.error = null;
			try {
				const res = await fetch(
					`https://backend.raycast.com/api/v1/store_listings/search?q=${encodeURIComponent(value)}&per_page=${this.perPage}`
				);
				if (!res.ok) throw new Error(`Search failed: ${res.status}`);
				const parsed = PaginatedExtensionsResponseSchema.parse(await res.json());
				this.searchResults = parsed.data;
				this.selectedIndex = 0;
			} catch (e: unknown) {
				this.error = e instanceof Error ? e.message : 'Unknown error';
				console.error(e);
				this.searchResults = [];
			} finally {
				clearTimeout(this.#loadingDebounceTimer);
				if (value === this.#_searchText) {
					this.isLoading = false;
					this.isSearching = false;
				}
			}
		}, 300);
	}

	async #fetchInitialData() {
		try {
			this.isLoading = true;
			this.error = null;
			const [storeRes, featuredRes, trendingRes] = await Promise.all([
				fetch(`https://backend.raycast.com/api/v1/store_listings?page=1&per_page=${this.perPage}`),
				fetch('https://backend.raycast.com/api/v1/extensions/featured'),
				fetch('https://backend.raycast.com/api/v1/extensions/trending')
			]);

			if (!storeRes.ok) throw new Error(`Store fetch failed: ${storeRes.status}`);
			const storeParsed = PaginatedExtensionsResponseSchema.parse(await storeRes.json());
			this.extensions = storeParsed.data;
			this.currentPage = 1;
			this.hasMore = storeParsed.data.length === this.perPage;

			if (featuredRes.ok) {
				const featuredParsed = PaginatedExtensionsResponseSchema.parse(await featuredRes.json());
				this.featuredExtensions = featuredParsed.data;
			}

			if (trendingRes.ok) {
				const trendingParsed = PaginatedExtensionsResponseSchema.parse(await trendingRes.json());
				this.trendingExtensions = trendingParsed.data;
			}
		} catch (e: unknown) {
			this.error = e instanceof Error ? e.message : 'Unknown error';
			console.error(e);
		} finally {
			this.isLoading = false;
		}
	}

	loadMore = async () => {
		if (
			this.isFetchingMore ||
			!this.hasMore ||
			this.searchText ||
			this.selectedCategory !== 'All Categories'
		) {
			return;
		}

		this.isFetchingMore = true;
		const nextPage = this.currentPage + 1;

		try {
			const res = await fetch(
				`https://backend.raycast.com/api/v1/store_listings?page=${nextPage}&per_page=${this.perPage}`
			);
			if (!res.ok) throw new Error('Failed to fetch more extensions');
			const parsed = PaginatedExtensionsResponseSchema.parse(await res.json());

			if (parsed.data.length < this.perPage) {
				this.hasMore = false;
			}

			const allExtensions = [...this.extensions, ...parsed.data];
			this.extensions = [...new Map(allExtensions.map((item) => [item.id, item])).values()];
			this.currentPage = nextPage;
		} catch (e) {
			console.error('Error loading more extensions:', e);
			this.hasMore = false;
		} finally {
			this.isFetchingMore = false;
		}
	};
}

export const extensionsStore = new ExtensionsStore();
