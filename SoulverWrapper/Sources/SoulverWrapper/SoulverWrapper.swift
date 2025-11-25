import SoulverCore
import Foundation

struct SoulverResult: Codable {
    let value: String
    let type: String
    let error: String?

    init(value: String, type: String, error: String? = nil) {
        self.value = value
        self.type = type
        self.error = error
    }
}

@MainActor
private var globalCalculator: Calculator?

@MainActor
@_cdecl("initialize_soulver")
public func initialize_soulver(resourcesPath: UnsafePointer<CChar>) {
    guard globalCalculator == nil else {
        return
    }

    let pathString = String(cString: resourcesPath)
    let resourcesURL = URL(fileURLWithPath: pathString)
    
    guard SoulverCore.ResourceBundle(url: resourcesURL) != nil else {
        print("[SoulverWrapper] Failed to create SoulverCore.ResourceBundle from path: \(pathString)")
        return
    }
    
    var customization = EngineCustomization.standard
    
    let currencyProvider = RaycastCurrencyProvider()
    customization.currencyRateProvider = currencyProvider
    
    currencyProvider.startUpdating()
    
    globalCalculator = Calculator(customization: customization)
}

@MainActor
@_cdecl("evaluate")
public func evaluate(expression: UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>? {
    let encoder = JSONEncoder()

    guard let calculator = globalCalculator else {
        let errorMsg = "SoulverCore not initialized"
        print("[SoulverWrapper] \(errorMsg)")
        let errorResult = SoulverResult(value: "", type: "error", error: errorMsg)
        if let jsonData = try? encoder.encode(errorResult), let jsonString = String(data: jsonData, encoding: .utf8) {
            return strdup(jsonString)
        }
        return nil
    }

    let swiftExpression = String(cString: expression)
    
    if swiftExpression.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty {
        let emptyResult = SoulverResult(value: "", type: "none", error: nil)
        if let jsonData = try? encoder.encode(emptyResult), let jsonString = String(data: jsonData, encoding: .utf8) {
            return strdup(jsonString)
        }
        return nil
    }

    let result = calculator.calculate(swiftExpression)
    
    if result.isEmptyResult {
         let emptyResult = SoulverResult(value: "", type: "none", error: nil)
         if let jsonData = try? encoder.encode(emptyResult), let jsonString = String(data: jsonData, encoding: .utf8) {
             return strdup(jsonString)
         }
         return nil
    }

    let soulverResult = SoulverResult(value: result.stringValue, type: formatResult(result: result.evaluationResult, customization: calculator.customization))
    
    if let jsonData = try? encoder.encode(soulverResult),
       let jsonString = String(data: jsonData, encoding: .utf8) {
        return strdup(jsonString)
    }
    
    let errorResult = SoulverResult(value: "", type: "error", error: "Failed to encode Soulver result to JSON.")
    if let jsonData = try? encoder.encode(errorResult), let jsonString = String(data: jsonData, encoding: .utf8) {
        return strdup(jsonString)
    }
    return nil
}

@_cdecl("free_string")
public func free_string(ptr: UnsafeMutablePointer<CChar>?) {
    free(ptr)
}