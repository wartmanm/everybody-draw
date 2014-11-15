#!/usr/bin/env python

from __future__ import print_function

import os
import subprocess
from itertools import chain
import sys
import argparse
import json
import re

platformpath = '{ANDROID_NDK_ROOT}/platforms/{PLATFORM_NAME}/arch-arm'.format(**os.environ)
default_header_path = '{}/usr/include/'.format(platformpath)
includes = set([
  default_header_path,
  '{ANDROID_NDK_ROOT}/toolchains/arm-linux-androideabi-4.6/prebuilt/linux-x86_64/lib/gcc/arm-linux-androideabi/4.6/include/'.format(**os.environ)
])

prelude_lints = [
    'unused_attributes',
    'unused_imports',
    'non_camel_case_types',
    'non_snake_case',
    'non_upper_case_globals',
]

builtins_name = 'bindgen_builtins'
fnptr_re = re.compile('::std::option::Option<(extern "C" fn.*?[^-])>', re.DOTALL)

prefix = None

class Binding:
  def __init__(self, **kwargs):
    self.path = kwargs['path']
    self.match = kwargs.get('match') or []
    self.deps = kwargs.get('deps') or []
    self.deps.append(builtins_name)
    self.remove_fnptr_opts = kwargs.get('remove_fnptr_opts') or False
    self.preexisting = False
    self.root = kwargs.get('root') or default_header_path
  def is_preexisting(self):
    return self.preexisting

class Module:
  def __init__(self, basename):
    self.path = basename
    self.mods = set()
  def is_preexisting(self):
    return all(map(lambda x: x.is_preexisting(), self.mods))

def flatten(x):
  return list(chain.from_iterable(x))

def run_bindgen(args, outfile):
  allargs = ['bindgen'] + args
  #print('running ' + ' '.join(allargs))
  p = subprocess.Popen(allargs, stdout=subprocess.PIPE)
  return p.communicate()[0]

def append_allow_prelude(outfile):
  for lint in prelude_lints:
    outfile.write('#![allow({})]\n'.format(lint))

def gen_bindings(binding):
  dest = joinprefix(binding.path + '.rs')
  if os.path.exists(dest):
    binding.preexisting = True
    return
  matches = binding.match
  matches = [['-match', x] for x in matches] if matches else []
  includeargs = [['-I', x] for x in includes]
  header = os.path.join(binding.root, binding.path + '.h')

  print('writing bindings for {}'.format(binding.path))

  destdir = os.path.dirname(dest)
  if not os.path.exists(destdir):
    os.makedirs(destdir)
  with open(dest, 'w') as outfile:
    append_allow_prelude(outfile)
    for dep in binding.deps:
      outfile.write('use {}::*;\n'.format(dep))
    args = flatten(includeargs + matches + [[header]])
    stdout = run_bindgen(args, outfile)
    if binding.remove_fnptr_opts:
      stdout = re.sub(fnptr_re, '\\1', stdout)
    outfile.write(stdout)

def gen_builtins():
  dest = joinprefix(builtins_name + '.rs')
  if os.path.exists(dest):
    return
  with open(dest, 'w') as outfile:
    append_allow_prelude(outfile)
    outfile.flush()
    p = subprocess.Popen('bindgen -builtins -E -'.split(), stdin=subprocess.PIPE, stdout=outfile)
    p.communicate()

def gen_modfile(path, mod):
  print('writing bindings for {}'.format(path))
  if mod.is_preexisting():
    return
  with open(joinprefix(path, 'mod.rs'), 'w') as outfile:
    for submod in mods.mods:
      relative_mod = os.path.relpath(submod.path, path)
      outfile.write('pub mod {};\n'.format(relative_mod.replace('/', '::')))

def gathermods(bindings):
  dirs = {}
  for binding in bindings:
    modpath = binding.path
    submod = binding
    while True:
      parent = os.path.dirname(modpath)
      if parent in ['/', '']: break
      mod = dirs.setdefault(parent, Module(parent))
      mod.mods.add(submod)
      modpath = parent
      submod = mod
  return dirs

def sloppy_delete(path):
  try:
    os.remove(path)
  except OSError: return False
  else: return True

def joinprefix(*args):
  return os.path.join(prefix, *args)

def add_include_roots(bindings):
  for binding in bindings:
    includes.add(binding.root)

if __name__ == '__main__':
  parser = argparse.ArgumentParser(description='Generate android bindings using bindgen.')
  parser.add_argument('--prefix', help='prefix for rust source directory', default='.')
  parser.add_argument('source', help='source file for bindings')
  parser.add_argument('mode', choices=['build','clean'], help='generate or delete bindings')
  args = parser.parse_args()
  prefix = args.prefix
  with open(args.source) as bindingfile:
    bindings = [Binding(**x) for x in json.load(bindingfile)]

  if args.mode == 'build':
    add_include_roots(bindings)
    gen_builtins()
    for binding in bindings:
      gen_bindings(binding)
    for path, mods in gathermods(bindings).items():
      gen_modfile(path, mods)

  elif args.mode == 'clean':
    deletes = [x.path for x in bindings] + [builtins_name]
    for path in [x.path for x in bindings] + [builtins_name]:
      sloppy_delete(joinprefix(path + '.rs'))
    for path, mods in gathermods(bindings).items():
      sloppy_delete(joinprefix(path, 'mod.rs'))


