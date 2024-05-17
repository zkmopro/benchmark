import Foundation
import JavaScriptCore

/// An analyzer of sentiments
class SentimentAnalyzer {
  /// Singleton instance. Much more resource-friendly than creating multiple new instances.
  static let shared = SentimentAnalyzer()
  private let vm = JSVirtualMachine()
  private let context: JSContext

  private init() {
    let jsCode = """
      // From https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Math/random
      function randomNumber(min, max) {
          min = Math.ceil(min);
          max = Math.floor(max);
          //The maximum is inclusive and the minimum is inclusive
          return Math.floor(Math.random() * (max - min + 1)) + min;
      }

      function analyze(sentence) {
          return randomNumber(-5, 5);
      }

      async function fetchData() {
          return new Promise((resolve, reject) => {
              setTimeout(() => {
                  resolve("Data fetched");
              }, 2000);
          });
      }
      """

    // Create a new JavaScript context that will contain the state of our evaluated JS code.
    context = JSContext(virtualMachine: vm)

    // Evaluate the JS code that defines the functions to be used later on.
    context.evaluateScript(
      jsCode,
      withSourceURL: URL(
        string:
          "https://gist.githubusercontent.com/vivianjeng/374c0fd6e45910e916fc5c2bba704131/raw/f008bfd687797bb6b768a3670d96e6bea5a7cb12/snark.min.js"
      )
    )

  }

  /// Analyze the sentiment of a given English sentence.
  /// - Parameters:
  ///     - sentence: The sentence to analyze
  /// - Returns : The sentiment score
  func analyze(_ sentence: String) async -> Int {
    if let result = context.globalObject.invokeMethod("analyze", withArguments: [sentence]) {
      return Int(result.toInt32())
    }
    return 0
  }

  func fetchData() async -> String {
    print("fetchData")
    let handleResult: @convention(block) (JSValue) -> String = { result in
      print("Success: \(result.toString() ?? "No result")")
        return result.toString()
    }
    let handleResultJSValue = JSValue(object: handleResult, in: context)
    if let promise = context.globalObject.invokeMethod("fetchData", withArguments: []) {
        promise.invokeMethod("then", withArguments: [handleResultJSValue])
    }
    return "0"
  }

  /// Return an emoji for the given sentiment score.
  /// - Parameters:
  ///     - score: The sentiment score
  /// - Returns: String with a single emoji character
  func emoji(forScore score: Int) -> String {
    switch score {
    case 5...Int.max:
      return "😍"
    case 4:
      return "😃"
    case 3:
      return "😊"
    case 2, 1:
      return "🙂"
    case -1, -2:
      return "🙁"
    case -3:
      return "☹️"
    case -4:
      return "😤"
    case Int.min...(-5):
      return "😡"
    default:
      return "😐"
    }
  }
}
