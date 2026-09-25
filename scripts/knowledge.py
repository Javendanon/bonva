#!/usr/bin/env python3
"""Local book ingestion and exact, deterministic retrieval. Never assigns code scores."""
import argparse
import hashlib
import json
import posixpath
import re
import sys
import unicodedata
import zipfile
from html.parser import HTMLParser
from pathlib import Path
from urllib.parse import unquote
from xml.etree import ElementTree as ET

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "knowledge/sources/manifest.json"


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def emit(value):
    print(json.dumps(value, ensure_ascii=False, sort_keys=True, indent=2))


def write_json(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")


def records():
    return json.loads(MANIFEST.read_text(encoding="utf-8")) if MANIFEST.exists() else []


def get_record(book):
    return next((r for r in records() if r["id"] == book), None)


def units(record):
    return json.loads((ROOT / record["extracted"]).read_text(encoding="utf-8"))


def normalize(text):
    return " ".join(unicodedata.normalize("NFKC", text).casefold().split())


class TextParser(HTMLParser):
    def __init__(self):
        super().__init__()
        self.parts = []
        self.hidden = 0

    def handle_starttag(self, tag, attrs):
        if tag in ("script", "style"):
            self.hidden += 1
        if tag in ("p", "div", "br", "li", "h1", "h2", "h3", "tr"):
            self.parts.append("\n")

    def handle_endtag(self, tag):
        if tag in ("script", "style"):
            self.hidden = max(0, self.hidden - 1)
        if tag in ("p", "div", "li", "h1", "h2", "h3", "tr"):
            self.parts.append("\n")

    def handle_data(self, data):
        if not self.hidden:
            self.parts.append(data)


def extract(path):
    if path.suffix.lower() == ".pdf":
        import pymupdf
        with pymupdf.open(path) as doc:
            rows = [{"unit": i + 1, "text": p.get_text(sort=True)} for i, p in enumerate(doc)]
        return rows, {"name": "PyMuPDF", "version": pymupdf.VersionBind, "sort": True}
    if path.suffix.lower() != ".epub":
        raise ValueError("Supported formats: PDF and EPUB")
    with zipfile.ZipFile(path) as archive:
        container = ET.fromstring(archive.read("META-INF/container.xml"))
        package = next(n.attrib["full-path"] for n in container.iter() if n.tag.endswith("}rootfile"))
        opf = ET.fromstring(archive.read(package))
        items = {n.attrib["id"]: n.attrib["href"] for n in opf.iter() if n.tag.endswith("}item")}
        rows = []
        for item in opf.iter():
            if not item.tag.endswith("}itemref"):
                continue
            href = items[item.attrib["idref"]]
            member = posixpath.normpath(posixpath.join(posixpath.dirname(package), unquote(href.split("#")[0])))
            parser = TextParser()
            parser.feed(archive.read(member).decode("utf-8-sig"))
            rows.append({"unit": len(rows) + 1, "href": href, "text": "".join(parser.parts)})
    return rows, {"name": "stdlib-epub-htmlparser", "version": 1, "python": sys.version.split()[0]}


def ingest(args):
    if not re.fullmatch(r"[a-z][a-z0-9_-]*", args.id):
        raise ValueError("Invalid id; use lowercase letters, digits, _ or -")
    path = Path(args.file).resolve()
    relative = path.relative_to(ROOT)
    existing = records()
    output = ROOT / f"knowledge/sources/{args.id}.json"
    if get_record(args.id) or output.exists():
        raise ValueError("ID already exists; use a new edition ID to preserve reading provenance")
    rows, extractor = extract(path)
    record = {"id": args.id, "title": args.title, "file": str(relative), "sha256": digest(path),
              "extracted": str(output.relative_to(ROOT)), "units": len(rows),
              "characters": sum(len(r["text"]) for r in rows),
              "empty_units": [r["unit"] for r in rows if not r["text"].strip()],
              "extractor": extractor, "reading_status": "unread", "validation_status": "unvalidated",
              "unit_kind": "pdf_physical_page" if path.suffix.lower() == ".pdf" else "epub_spine_item"}
    write_json(output, rows)
    record["extracted_sha256"] = digest(output)
    write_json(MANIFEST, sorted(existing + [record], key=lambda r: r["id"]))
    emit(record)


def verify():
    errors = []
    all_records = records()
    if len({r["id"] for r in all_records}) != len(all_records):
        errors.append("duplicate book ids")
    for record in all_records:
        try:
            rows = units(record)
            checks = {
                "source hash": digest(ROOT / record["file"]) == record["sha256"],
                "extraction hash": digest(ROOT / record["extracted"]) == record["extracted_sha256"],
                "unit count": len(rows) == record["units"],
                "locators": [r["unit"] for r in rows] == list(range(1, len(rows) + 1)),
                "characters": sum(len(r["text"]) for r in rows) == record["characters"],
                "empty units": [r["unit"] for r in rows if not r["text"].strip()] == record["empty_units"],
            }
            errors.extend(f"{record['id']}: {name}" for name, ok in checks.items() if not ok)
        except (OSError, ValueError, KeyError) as exc:
            errors.append(f"{record['id']}: {exc}")
    emit({"ok": not errors, "books": len(all_records), "errors": errors})
    return int(bool(errors))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)
    sub.add_parser("list")
    sub.add_parser("verify")
    show = sub.add_parser("show")
    show.add_argument("book")
    show.add_argument("unit", type=int)
    search = sub.add_parser("search")
    search.add_argument("query")
    search.add_argument("--book")
    search.add_argument("--limit", type=int, default=10)
    add = sub.add_parser("ingest")
    add.add_argument("id")
    add.add_argument("file")
    add.add_argument("--title", required=True)
    args = parser.parse_args()
    if args.command == "list":
        emit(sorted(records(), key=lambda r: r["id"]))
    elif args.command == "verify":
        return verify()
    elif args.command == "ingest":
        ingest(args)
    elif args.command == "show":
        record = get_record(args.book)
        if record is None or not 1 <= args.unit <= record["units"]:
            raise ValueError("Unknown book or out-of-range unit")
        emit({"book": args.book, "source_sha256": record["sha256"], **units(record)[args.unit - 1]})
    else:
        query = normalize(args.query)
        if not query or args.limit < 1:
            raise ValueError("Query must not be empty; limit must be positive")
        if args.book and get_record(args.book) is None:
            raise ValueError("Unknown book")
        matches = []
        for record in sorted(records(), key=lambda r: r["id"]):
            if args.book and record["id"] != args.book:
                continue
            for row in units(record):
                normalized = normalize(row["text"])
                position = normalized.find(query)
                if position >= 0:
                    matches.append({"book": record["id"], "unit": row["unit"],
                                    "source_sha256": record["sha256"],
                                    "excerpt_normalized": normalized[max(0, position - 100):position + len(query) + 250]})
        emit({"query": query, "total": len(matches), "matches": matches[:args.limit],
              "ordering": "book_id_then_unit", "method": "normalized_literal_substring"})
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (ValueError, OSError, KeyError, ImportError, zipfile.BadZipFile, ET.ParseError) as error:
        print(f"knowledge: {error}", file=sys.stderr)
        sys.exit(2)
