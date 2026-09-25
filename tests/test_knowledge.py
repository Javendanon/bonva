import contextlib
import importlib.util
import io
import json
import subprocess
import sys
import tempfile
import unittest
import zipfile
from argparse import Namespace
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SPEC = importlib.util.spec_from_file_location("knowledge", ROOT / "scripts/knowledge.py")
k = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(k)


class KnowledgeTests(unittest.TestCase):
    def cli(self, *args):
        return subprocess.run([sys.executable, str(ROOT / "scripts/knowledge.py"), *args], capture_output=True)

    def test_corpus_integrity_and_retrieval(self):
        result = self.cli("verify")
        self.assertEqual(result.returncode, 0, result.stdout)
        first = self.cli("search", "amortized", "--book", "pfds", "--limit", "3")
        second = self.cli("search", "amortized", "--book", "pfds", "--limit", "3")
        self.assertEqual(first.returncode, 0)
        self.assertEqual(first.stdout, second.stdout)
        self.assertGreater(json.loads(first.stdout)["total"], 0)
        page = json.loads(self.cli("show", "pfds", "66").stdout)
        self.assertEqual(page["unit"], 66)
        self.assertIn("Deque", page["text"])
        for args in [("show", "pfds", "0"), ("search", " "), ("search", "x", "--book", "missing")]:
            self.assertEqual(self.cli(*args).returncode, 2)

    def test_cli_listing_and_search_contract(self):
        listed = self.cli("list")
        self.assertEqual(listed.returncode, 0, listed.stderr)
        ids = [record["id"] for record in json.loads(listed.stdout)]
        self.assertEqual(ids, sorted(ids))
        searched = self.cli("search", " AMORTIZED ", "--book", "pfds", "--limit", "1")
        self.assertEqual(searched.returncode, 0, searched.stderr)
        result = json.loads(searched.stdout)
        self.assertEqual(result["query"], "amortized")
        self.assertEqual(len(result["matches"]), 1)
        self.assertGreater(result["total"], 1)
        self.assertEqual(result["matches"][0]["book"], "pfds")
        self.assertEqual(result["ordering"], "book_id_then_unit")

    def test_unsupported_format_has_explicit_error(self):
        with self.assertRaisesRegex(ValueError, "Supported formats: PDF and EPUB"):
            k.extract(Path("unsupported.txt"))

    def test_ingestion_preserves_order_and_provenance(self):
        original_root, original_manifest = k.ROOT, k.MANIFEST
        self.addCleanup(setattr, k, "ROOT", original_root)
        self.addCleanup(setattr, k, "MANIFEST", original_manifest)
        with tempfile.TemporaryDirectory() as tmp:
            k.ROOT = Path(tmp).resolve()
            k.MANIFEST = k.ROOT / "knowledge/sources/manifest.json"
            book = k.ROOT / "fixture.epub"
            with zipfile.ZipFile(book, "w") as z:
                z.writestr("META-INF/container.xml", '<container xmlns="urn:oasis:names:tc:opendocument:xmlns:container"><rootfiles><rootfile full-path="OPS/book.opf"/></rootfiles></container>')
                z.writestr("OPS/book.opf", '<package xmlns="http://www.idpf.org/2007/opf"><manifest><item id="b" href="second.xhtml"/><item id="a" href="first.xhtml"/></manifest><spine><itemref idref="a"/><itemref idref="b"/></spine></package>')
                z.writestr("OPS/first.xhtml", '<html><body><p>First <em>visible</em> &amp; café</p><script>hidden</script></body></html>')
                z.writestr("OPS/second.xhtml", '<html><body><p>Second</p></body></html>')
            args = Namespace(id="fixture", file=str(book), title="Fixture")
            with contextlib.redirect_stdout(io.StringIO()):
                k.ingest(args)
                self.assertEqual(k.verify(), 0)
            record = k.get_record("fixture")
            rows = k.units(record)
            self.assertEqual(record["reading_status"], "unread")
            self.assertEqual([r["href"] for r in rows], ["first.xhtml", "second.xhtml"])
            self.assertIn("café", rows[0]["text"])
            self.assertNotIn("hidden", rows[0]["text"])
            snapshot = k.MANIFEST.read_bytes()
            with self.assertRaises(ValueError):
                k.ingest(args)
            self.assertEqual(snapshot, k.MANIFEST.read_bytes())
            book.write_bytes(book.read_bytes() + b"changed")
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(k.verify(), 1)


if __name__ == "__main__":
    unittest.main()
