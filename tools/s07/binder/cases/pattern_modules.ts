declare module "*.style" with { type: "css" } { const css: string; export default css; }
declare module "*.style" with { type: "text" } { const text: string; export default text; }
declare module "*.asset" { export const value: unknown; }
declare module "bad*pattern*" { export const bad: never; }
namespace Outer { export namespace Inner { export interface T {} export const value = 1; } }
namespace Outer { import Alias = Inner; export { Alias }; }
