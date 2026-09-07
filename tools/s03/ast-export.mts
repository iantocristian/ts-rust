// This adapter serializes resolver results; it does not interpret ast.json.
import * as fs from "node:fs";
import * as path from "node:path";
import { pathToFileURL } from "node:url";

const [toolingRoot, output] = process.argv.slice(2);
if (!toolingRoot || !output) throw new Error("usage: ast-export.mts TOOLING_ROOT OUTPUT");
const { api } = await import(pathToFileURL(path.join(toolingRoot, "tools/scripts/tsc/schema.ts")).href);

function type(type: any): any {
    switch (type.kind) {
        case "primitive": return { kind: type.kind, name: type.name };
        case "node": return { kind: type.kind, name: type.name };
        case "kind": return { kind: type.kind, name: type.name };
        case "list": return { kind: type.kind, list: type.listKind, element: normalizedType(type.elementType) };
        case "union": return { kind: type.kind, baseKind: type.baseKind(), types: type.types.map(normalizedType) };
        case "alias": return { kind: type.kind, name: type.name, resolved: normalizedType(type.resolved) };
        case "typeParameter": return { kind: type.kind, name: type.name, resolved: normalizedType(type.constraint) };
        default: throw new Error(`unknown resolver type ${type.kind}`);
    }
}
const normalizedType = type;

function member(member: any): any {
    return {
        name: member.name,
        type: normalizedType(member.type),
        optional: member.optional,
        private: member.private,
        inherited: member.inherited,
        goOnly: member.goOnly,
        noGo: member.noGo,
        noTS: member.noTS,
        noFactory: member.noFactory,
        kindParameter: member.isKindParam(),
        child: member.isChild(),
        visit: member.visit ?? null,
        bitmask: member.bitmask ?? null,
        declaredType: normalizedType(member.declaredType),
    };
}

// Follow the inheritance graph the resolver exposes. A concrete member's
// resolved type and optionality narrow its base storage field where applicable.
function storageFields(node: any): any[] {
    const fields = new Map<string, any>();
    for (const base of node.extends) {
        for (const field of storageFields(base)) fields.set(field.name, field);
    }
    for (const field of node.fields) {
        if (!field.noGo) fields.set(field.name, member(field));
    }
    for (const field of node.members) {
        if (!field.noGo && !field.isKindParam()) fields.set(field.name, member(field));
    }
    return [...fields.values()];
}

function baseTypes(node: any): string[] {
    return [...new Set(node.extends.flatMap((base: any) => [base.name, ...baseTypes(base)]))] as string[];
}

const normalized = {
    version: 1,
    runtimeVersion: 1,
    kinds: api.kindElements().filter((element: any) => element.name)
        .map((element: any, value: number) => ({ name: element.name, value })),
    markers: api.kindMarkers().map((marker: any) => ({ name: marker.name, value: api.resolveKindMarkerValue(marker.name) })),
    kindAliases: api.kindAliases().map((alias: any) => ({
        name: alias.name,
        kinds: api.expandKindAliasMembers(alias.name).map((kind: any) => kind.name),
    })),
    kindGuards: api.kindGuards().map((guard: any) => ({
        alias: guard.aliasName,
        form: guard.type,
        first: guard.first ?? null,
        last: guard.last ?? null,
        kinds: api.expandKindAliasMembers(guard.aliasName).map((kind: any) => kind.name),
    })),
    bases: api.bases().map((base: any) => ({ name: base.name, extends: base.extendsKeys, fields: base.fields.map(member) })),
    nodes: api.nodes().map((node: any) => ({
        name: node.name,
        kinds: node.allKinds().map((kind: any) => kind.name),
        handWritten: node.handWritten,
        handWrittenVisitor: node.handWrittenVisitor,
        syntaxKindName: node.syntaxKindName,
        kindAliases: node.kindAliases,
        kindType: normalizedType(node.kindType),
        kindTypes: node.kindTypes().map((kind: any) => kind.name),
        multiKind: node.isMultiKind(),
        generateSubtreeFacts: node.generateSubtreeFacts,
        baseTypes: baseTypes(node),
        extends: node.extendsKeys,
        members: node.members.map(member),
        fields: storageFields(node),
    })),
};
fs.writeFileSync(output, `${JSON.stringify(normalized, null, 2)}\n`);
