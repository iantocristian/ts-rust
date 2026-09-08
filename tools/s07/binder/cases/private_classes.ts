export class First { #x = 1; static #y = 2; get #value() { return this.#x; } set #value(v) { this.#x = v; } method(other: First) { return #x in other && other.#x; } }
export class Second extends First { #x = 3; static { this.name; } method() { return this.#x; } }
class Duplicate { #x; #x; constructor(public x: number) {} }
