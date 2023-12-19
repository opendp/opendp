import os
from opendp._lib import proven


def test_proven(monkeypatch):
  def fake():
    '''
    Fake doc string

    [(Proof Document)](url-is-replaced)
    '''
  
  # Proof doc does not actually exist, so fake it:
  monkeypatch.setattr(os.path, 'exists', lambda _: True)

  proven_fake = proven(fake)
  assert (
    '[(Proof Document)](https://docs.opendp.org/'
    'en/latest/proofs/python/src/opendp/extrinsics/'
    '../../../test/url-is-replaced)'
  ) in proven_fake.__doc__
