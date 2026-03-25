# Super (SPL) Manual

Super (SPL) is a hybrid polyglot language that brings together the rigidity and strong typing of Java, the rapid class creation of Python (dataclasses), and the dynamic object manipulation of JavaScript.

## 1. Syntax Basics
Super supports `let` (immutable) and `var` (mutable) variable declarations. All variables must be typed.

```java
var x: int = 10;
let nome: string = "Super";
```

## 2. Loops (The Omni-Loop)
Super supports parsing multiple styles of loops:
- **C/Java style:** `for (var i: int = 0; i < 10; i = i + 1) { ... }`
- **JavaScript style:** `for (let item of list) { ... }`
- **Python/PHP style:** `for item in list { ... }`
- **While loops:** `while (x < 10) { ... }`

## 3. Object-Oriented Programming (OOP)
### Dataclasses
Dataclasses (like Python) are quickly declared and provide an implicit constructor.

```java
type Animal(dataclass) {
    nome: string;
    idade: int;
}

let dog: object = new Animal("Rex", 5);
```

### Classes
Classes allow explicit method definitions and mutability toggles.

```java
class Calculator {
    var result: int;
    
    fn add(a: int, b: int) -> int {
        return a + b;
    }
}

let calc: object = new Calculator(0);
let ans: int = calc.add(10, 20);
```

## 4. Imports (.super files)
Super allows you to import definitions from another file easily:

```java
import "math.super";
// math.super functions and classes are now available
```

## 5. The Ministers (FFI)
Super allows loading Python, JavaScript, Java, and C/C++ libraries through the 'Ministers'. The engine initializes them upon REPL boot. Currently, they stand as foundational dummy implementations in `src/ministers/` utilizing the latest high-performance crates.
