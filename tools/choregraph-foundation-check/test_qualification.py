"""Synthetic qualification policy tests; no production mutation or compilation."""

import unittest

from octet_audit import source_exemptions, verify_baseline
from proof_audit import CORE, inspect_source, verify_inventory, verify_summary, verify_theorems
from rust_tokens import AuditError, lex


class ProofAudit(unittest.TestCase):
    def inspect(self, text, path=CORE + "/synthetic.rs"):
        return inspect_source(path, lex(text))

    def test_comments_and_literals_are_not_source_items(self):
        _, functions = self.inspect('''
            /* #[verifier::external_body] /* nested */ */
            const EXAMPLE: &str = r##"assume false; admit; fn invented() {}"##;
            verus! { proof fn actual() ensures true { assert(true); } }
        ''')
        self.assertEqual(functions, {"actual": "proof"})

    def test_rejects_attributes_outside_policy(self):
        for attribute in ("#[verifier /* comment */ :: unknown]", "#[verifier::]",
                          "#[cfg_attr(test, allow(unused))]", "#[cfg(unix)]",
                          "#[derive(unreviewed::Example)]", "#[derive]", "#![trigger]"):
            with self.subTest(attribute=attribute), self.assertRaises(AuditError):
                self.inspect(f'verus! {{ {attribute} proof fn lemma() ensures true {{}} }}')

    # r[verify choregraph.proof.single_step.readiness_completeness_negative]
    def test_rejects_assumptions_and_unverified_functions(self):
        for text in ('// verus! {\n fn ordinary() {}',
                     'const EXAMPLE: &str = "verus! {"; fn ordinary() {}',
                     'verus! { proof fn p() ensures true { fn unrelated() {} } }'):
            with self.subTest(text=text), self.assertRaisesRegex(AuditError, "outside a checked"):
                self.inspect(text)
        for name in ("admit", "assume", "axiom", "assume_specification"):
            with self.subTest(name=name), self.assertRaisesRegex(AuditError, "forbidden proof trust token"):
                self.inspect(f'verus! {{ proof fn p() ensures true {{ let _ = {name}; }} }}')

    def test_test_scope_is_exact(self):
        text = "#[cfg(test)] mod tests { fn local() {} }"
        self.assertEqual(self.inspect(text, CORE + ".rs"), ({}, {}))
        with self.assertRaisesRegex(AuditError, "unreviewed conditional"):
            self.inspect(text)
        with self.assertRaisesRegex(AuditError, "outside a checked"):
            self.inspect(text + " fn outside() {}", CORE + ".rs")

    def test_proof_module_scope(self):
        children, _ = self.inspect('#[path = "child.rs"] pub(crate) mod child;')
        self.assertEqual(children, {"child": CORE + "/child.rs"})
        for text in ('#[path = "../other.rs"] mod child;', 'mod child;',
                     'include!("child.rs");', '"#[path = child.rs]"; mod child;'):
            with self.subTest(text=text), self.assertRaises(AuditError):
                self.inspect(text)

    def test_removed_inventory_entry(self):
        parsed = {"file.rs": ({}, {"helper": "proof"})}
        with self.assertRaisesRegex(AuditError, "inventory changed"):
            verify_inventory(parsed, {"file.rs": {"modules": {}, "functions": {"helper": "proof", "missing": "exec"}}})

    def test_reconciles_names_modes_and_labels(self):
        mapping = {"named property": {"crate::checked::lemma": "proof"}}
        manifest = {"theorems": ["named property"]}
        functions = {"crate::checked::lemma": "proof"}
        verify_theorems(manifest, mapping, functions)
        for labels in ([], ["only a description"], ["named property", "named property"]):
            with self.subTest(labels=labels), self.assertRaisesRegex(AuditError, "labels changed"):
                verify_theorems({"theorems": labels}, mapping, functions)
        for names in ({"crate::other::lemma": "proof"}, {"crate::checked::lemma": "spec"}, {}):
            with self.subTest(names=names), self.assertRaisesRegex(AuditError, "expected checked proof item"):
                verify_theorems(manifest, mapping, names)

    def test_exact_nonzero_summary(self):
        stdout = "verification results:: 2 verified, 0 errors\n"
        verify_summary({"verified_count": 2}, stdout)
        for count in (0, -1, True, "2", 3):
            with self.subTest(count=count), self.assertRaises(AuditError):
                verify_summary({"verified_count": count}, stdout)
        for text in (stdout + stdout, stdout[:-1] + " trailing text", "prefix " + stdout):
            with self.subTest(text=text), self.assertRaises(AuditError):
                verify_summary({"verified_count": 2}, text)


class OctetAudit(unittest.TestCase):
    def collect(self, text):
        return source_exemptions(lex(text), {"magic_number"})

    def test_does_not_count_comments_strings_or_other_lints(self):
        self.assertEqual(self.collect('''
            // #[allow(magic_number)]
            const EXAMPLE: &str = "#[allow(magic_number)]";
            #[allow(dead_code)] fn ordinary() {}
        '''), {})

    def test_distinguishes_scope_and_conditional_exemptions(self):
        source = '''
            #![allow(magic_number)]
            #[allow(octet::magic_number, reason = "explanation")]
            fn ordinary() {}
            #[cfg_attr(test, expect(magic_number))] mod tests {}
            verus! { #[allow(magic_number)] fn checked() {} }
        '''
        self.assertEqual(self.collect(source), {
            "file allow magic_number": 1, "item allow magic_number": 1,
            "conditional-item expect magic_number": 1, "verus-item allow magic_number": 1,
        })

    def test_no_blanket_or_unreviewed_growth(self):
        for text in ('#![allow(warnings)]', '#[cfg_attr(test, warn(octet::all))]'):
            with self.subTest(text=text), self.assertRaisesRegex(AuditError, "blanket"):
                self.collect(text)
        baseline = {"pkg/a.rs": {"item allow magic_number": 1}}
        for file, source in (("pkg/b.rs", "#[allow(magic_number)]"),
                             ("pkg/a.rs", "#![allow(magic_number)]"),
                             ("pkg/a.rs", "#[allow(magic_number)] #[allow(magic_number)]")):
            with self.subTest(file=file, source=source), self.assertRaisesRegex(AuditError, "policy review required"):
                verify_baseline({file: self.collect(source)}, baseline)
        with self.assertRaisesRegex(AuditError, "lower the baseline"):
            verify_baseline({}, baseline)


if __name__ == "__main__":
    unittest.main()
