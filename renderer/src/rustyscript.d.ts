declare namespace rustyscript {
  declare namespace async_functions {
    function showToast(toast: import("./index").ToastOptions): Promise<void>;
    function updateTree(tree: any): Promise<void>;
  }
}
