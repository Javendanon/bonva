"""Validator for the explicit JSON Schema subset used by evaluator.json.

Not a general JSON Schema implementation. Semantic invariants live in engine.py.
"""
import json
import math
from pathlib import Path

SCHEMA = json.loads((Path(__file__).with_name("evaluator.json")).read_text())


def validate(kind, value):
    def visit(spec, data, path):
        if "$ref" in spec:
            return visit(SCHEMA["$defs"][spec["$ref"].removeprefix("#/$defs/")], data, path)
        types = spec.get("type", [])
        if isinstance(types, str):
            types = [types]
        matches = {"null": data is None, "boolean": type(data) is bool,
                   "integer": type(data) is int, "number": type(data) in (int, float),
                   "string": isinstance(data, str), "object": isinstance(data, dict), "array": isinstance(data, list)}
        if types and not any(matches[t] for t in types):
            raise ValueError(f"{path}: expected {types}")
        if "const" in spec and data != spec["const"]:
            raise ValueError(f"{path}: wrong constant/version")
        if "enum" in spec and data not in spec["enum"]:
            raise ValueError(f"{path}: unsupported value")
        if type(data) in (int, float):
            if not math.isfinite(data) or data < spec.get("minimum", -math.inf) or data > spec.get("maximum", math.inf):
                raise ValueError(f"{path}: invalid number")
        if isinstance(data, dict):
            missing = set(spec.get("required", [])) - data.keys()
            if missing:
                raise ValueError(f"{path}: missing {sorted(missing)}")
            properties = spec.get("properties", {})
            if spec.get("additionalProperties") is False and set(data) - properties.keys():
                raise ValueError(f"{path}: unexpected properties")
            for key in data.keys() & properties.keys():
                visit(properties[key], data[key], f"{path}.{key}")
        if isinstance(data, list):
            if len(data) < spec.get("minItems", 0):
                raise ValueError(f"{path}: too few items")
            for i, item in enumerate(data):
                visit(spec.get("items", {}), item, f"{path}[{i}]")
    visit(SCHEMA["$defs"][kind], value, kind)
