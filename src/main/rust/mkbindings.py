#!/usr/bin/env python

from __future__ import print_function

import os
import subprocess
from itertools import chain
import sys
import argparse
import json

platformpath = '{ANDROID_NDK_ROOT}/platforms/{PLATFORM_NAME}/arch-arm'.format(**os.environ)
includes = [
  '{}/usr/include'.format(platformpath),
  '{ANDROID_NDK_ROOT}/toolchains/arm-linux-androideabi-4.6/prebuilt/linux-x86_64/lib/gcc/arm-linux-androideabi/4.6/include/'.format(**os.environ)
]

prelude_lints = [
    'unused_attribute',
    'unused_imports',
    'non_camel_case_types',
    'non_snake_case',
    'non_uppercase_statics',
]

builtins_name = "bindgen_builtins"

prefix = None

def flatten(x):
  return list(chain.from_iterable(x))

def run_bindgen(args, outfile):
  allargs = ['bindgen'] + args
  #print('running ' + ' '.join(allargs))
  outfile.flush()
  subprocess.call(allargs, stdout=outfile)

def append_allow_prelude(outfile):
  for lint in prelude_lints:
    outfile.write('#![allow({})]\n'.format(lint))

def gen_bindings(binding):
  matches = binding.get('match')
  matches = [['-match', x] for x in matches] if matches else []
  includeargs = [['-I', x] for x in includes]
  header = '{}/usr/include/{path}.h'.format(platformpath, **binding)

  print('writing bindings for {path}'.format(**binding))

  with open(joinprefix(binding["path"] + '.rs'), 'w') as outfile:
    append_allow_prelude(outfile)
    for dep in (binding['deps'] if 'deps' in binding else []) + [builtins_name]:
      outfile.write('use {}::*;\n'.format(dep))
    args = flatten(includeargs + matches + [[header]])
    run_bindgen(args, outfile)

def gen_builtins():
  with open(joinprefix(builtins_name + '.rs'), 'w') as outfile:
    append_allow_prelude(outfile)
    outfile.flush()
    p = subprocess.Popen('bindgen -builtins -E -'.split(), stdin=subprocess.PIPE, stdout=outfile)
    p.communicate()

def gen_modfile(path, mods):
  print('writing bindings for {}'.format(path))
  with open(joinprefix(path, 'mod.rs'), 'w') as outfile:
    for mod in mods:
      outfile.write('pub mod {};\n'.format(mod.replace('/', '::')))

def gathermods(paths):
  dirs = {}
  for path in paths:
    while True:
      parent = os.path.dirname(path)
      if parent in ['/', '']: break
      mod = dirs.setdefault(parent, set())
      mod.add(os.path.basename(path))
      path = parent
  return dirs

def sloppy_delete(path):
  try:
    os.remove(path)
  except OSError: return False
  else: return True

def joinprefix(*args):
  return os.path.join(prefix, *args)

if __name__ == '__main__':
  parser = argparse.ArgumentParser(description='Generate android bindings using bindgen.')
  parser.add_argument('--prefix', help='prefix for rust source directory', default='.')
  parser.add_argument('source', help='source file for bindings')
  parser.add_argument('mode', choices=['build','clean'], help='generate or delete bindings')
  args = parser.parse_args()
  prefix = args.prefix
  with open(args.source) as bindingfile:
    bindings = json.load(bindingfile)

  if args.mode == 'build':
    gen_builtins()
    for binding in bindings:
      gen_bindings(binding)
    for path, mods in gathermods([x["path"] for x in bindings]).items():
      gen_modfile(path, mods)

  elif args.mode == 'clean':
    deletes = [x["path"] for x in bindings] + [builtins_name]
    for path in [x["path"] for x in bindings] + [builtins_name]:
      sloppy_delete(joinprefix(path + '.rs'))
    for path, mods in gathermods([x["path"] for x in bindings]).items():
      sloppy_delete(joinprefix(path, "mod.rs"))


