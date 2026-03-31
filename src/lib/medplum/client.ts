import { MedplumClient } from '@medplum/core'

let _client: MedplumClient | null = null

export function getMedplumClient(): MedplumClient {
  if (!_client) {
    _client = new MedplumClient({
      baseUrl: process.env.MEDPLUM_BASE_URL || 'https://api.medplum.com',
      clientId: process.env.MEDPLUM_CLIENT_ID,
      clientSecret: process.env.MEDPLUM_CLIENT_SECRET,
    })
  }
  return _client
}
