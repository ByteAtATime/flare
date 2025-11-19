declare namespace rustyscript {
  declare namespace async_functions {
    function showToast(toast: import("./index").ToastOptions): Promise<void>;
    function updateTree(tree: any): Promise<void>;
  }

  declare namespace functions {
    function cacheGet(namespace: string, key: string): string | null;
    function cacheSet(namespace: string, key: string, data: string): void;
    function cacheHas(namespace: string, key: string): boolean;
    function cacheRemove(namespace: string, key: string): boolean;
    function cacheClear(namespace: string): void;
    function cacheIsEmpty(namespace: string): boolean;
  }
}
