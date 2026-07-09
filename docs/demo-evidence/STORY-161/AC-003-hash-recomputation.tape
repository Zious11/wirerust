Output .worktrees/STORY-161/docs/demo-evidence/STORY-161/AC-003-hash-recomputation.gif
Output .worktrees/STORY-161/docs/demo-evidence/STORY-161/AC-003-hash-recomputation.webm

Require python3
Require git

Set FontFamily "Menlo"
Set FontSize 12
Set Width 1200
Set Height 750
Set Padding 20

Hide
Type "cat > /tmp/vp024_verify.py << 'PYEOF'"
Enter
Type "import hashlib, subprocess, re"
Enter
Type "def lf_norm(b): b=b.replace(b'\\r\\n',b'\\n'); return b.replace(b'\\r',b'\\n')"
Enter
Type "def extract_kani(content):"
Enter
Type "    p=content.find(b'#[cfg(kani)]')"
Enter
Type "    m=content.find(b'mod kani_proofs',p)"
Enter
Type "    br=content.find(b'{',m); d=1; i=br+1"
Enter
Type "    while i<len(content) and d>0:"
Enter
Type "        if content[i:i+1]==b'{': d+=1"
Enter
Type "        elif content[i:i+1]==b'}': d-=1"
Enter
Type "        i+=1"
Enter
Type "    return content[p:i]"
Enter
Type "a=subprocess.check_output(['git','show','6e9f2cc:src/analyzer/arp.rs'])"
Enter
Type "d=subprocess.check_output(['git','show','6e9f2cc:src/decoder.rs'])"
Enter
Type "sA=lf_norm(extract_kani(a)); sB=lf_norm(extract_kani(d))"
Enter
Type "hA=hashlib.sha256(sA).digest(); hB=hashlib.sha256(sB).digest()"
Enter
Type "final=hashlib.sha256(hA+hB).hexdigest()"
Enter
Type "stored='48296b21a5bbce59750e6210da8d55be8bf7d3d4a1ed6719088dd4ef59a2c8a5'"
Enter
Type "print(f'arp.rs section:  {len(sA)} bytes  sha256_A={hashlib.sha256(sA).hexdigest()[:16]}...')"
Enter
Type "print(f'decoder.rs section: {len(sB)} bytes sha256_B={hashlib.sha256(sB).hexdigest()[:16]}...')"
Enter
Type "print(f'final_hash: {final}')"
Enter
Type "print(f'stored:     {stored}')"
Enter
Type "print('MATCH:', final==stored)"
Enter
Type "PYEOF"
Enter
Show

Sleep 0.5s
Type "# AC-161-003: Recompute proof_file_hash from commit 6e9f2cc snapshot"
Enter
Sleep 0.3s
Type "# mini-Merkle: sha256( sha256_raw(arp.rs section) || sha256_raw(decoder.rs section) )"
Enter
Sleep 0.5s
Type "python3 /tmp/vp024_verify.py"
Enter
Sleep 6s

Type "# Confirm match with grep against VP-024 frontmatter"
Enter
Sleep 0.3s
Type "grep 'proof_file_hash' .factory/specs/verification-properties/vp-024-arp-parse-safety.md | head -1"
Enter
Sleep 2s

Sleep 2s
