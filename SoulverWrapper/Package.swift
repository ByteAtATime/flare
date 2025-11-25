// swift-tools-version: 6.1
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "SoulverWrapper",
    products: [
        // Products define the executables and libraries a package produces, making them visible to other packages.
        .library(
            name: "SoulverWrapper",
            type: .dynamic,
            targets: ["SoulverWrapper"]),
    ],
    dependencies: [],
    targets: [
        // Targets are the basic building blocks of a package, defining a module or a test suite.
        // Targets can depend on other targets in this package and products from dependencies.
        .target(
            name: "SoulverWrapper",
            dependencies: [],
            swiftSettings: [
                .unsafeFlags(["-I", "Vendor/SoulverCore-linux/Modules"])
            ],
            linkerSettings: [
                .linkedLibrary("SoulverCoreDynamic"),
                .unsafeFlags([
                    "-L", "Vendor/SoulverCore-linux",
                    "-Xlinker", "-rpath", "-Xlinker", "$ORIGIN/../../../Vendor/SoulverCore-linux"
                ])
            ]
        ),
        .testTarget(
            name: "SoulverWrapperTests",
            dependencies: ["SoulverWrapper"]
        ),
    ]
)
