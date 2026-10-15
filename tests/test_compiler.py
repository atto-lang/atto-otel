"""Host compiler-launch admission; fake Java does not qualify the guest library."""
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest


class CompilerAdmission(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        (self.root/'scripts').mkdir()
        shutil.copy(Path(__file__).resolve().parents[1]/'scripts/compiler.sh',self.root/'scripts/compiler.sh')
        self.source = self.root/'compiler'
        self.source.mkdir()
        self.git('init','-q')
        (self.source/'.gitignore').write_text('target/\n')
        (self.source/'tracked').write_text('original')
        self.git('add','.')
        self.git('-c','user.name=Fixture','-c','user.email=fixture@example.invalid','commit','-qm','fixture')
        (self.root/'SOURCE_REF').write_text(self.git('rev-parse','HEAD'))
        (self.source/'target').mkdir()
        (self.source/'target/classpath.txt').write_text('fixture-classpath')
        self.marker=self.root/'java-called'
        java=self.root/'java'
        java.write_text('#!/bin/sh\ntouch "$JAVA_MARKER"\n')
        java.chmod(0o755)
        self.env={**os.environ,'ORESLANG_SOURCE_DIR':str(self.source),'ORES_JAVA':str(java),'JAVA_MARKER':str(self.marker)}

    def git(self,*args):
        return subprocess.check_output(['git','-C',str(self.source),*args],text=True).strip()

    def launch(self):
        return subprocess.run(['bash','scripts/compiler.sh','test.ores'],cwd=self.root,env=self.env,capture_output=True,text=True)

    def rejected(self,message):
        result=self.launch()
        self.assertNotEqual(result.returncode,0)
        self.assertIn(message,result.stderr)
        self.assertFalse(self.marker.exists())

    def test_exact_clean_pin_launches(self):
        self.assertEqual(self.launch().returncode,0)
        self.assertTrue(self.marker.exists())

    def test_wrong_and_floating_pins_rejected(self):
        (self.root/'SOURCE_REF').write_text('0'*40)
        self.rejected('revision differs')
        (self.root/'SOURCE_REF').write_text('main')
        self.rejected('exact compiler commit')

    def test_dirty_and_untracked_source_rejected(self):
        (self.source/'tracked').write_text('developer edits')
        self.rejected('must be clean')
        (self.source/'tracked').write_text('original')
        (self.source/'untracked').write_text('extra source')
        self.rejected('must be clean')

    def test_nested_directory_cannot_impersonate_checkout(self):
        child=self.source/'nested'
        child.mkdir()
        self.env['ORESLANG_SOURCE_DIR']=str(child)
        self.rejected('its own Git checkout')


if __name__=='__main__':
    unittest.main()
