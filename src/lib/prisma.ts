import { PrismaPg } from '@prisma/adapter-pg';
import { PrismaClient } from '../../generated/prisma/client';
import { DATABASE_URL } from '$env/static/private';

const adapter = new PrismaPg({ connectionString: `${DATABASE_URL}` });

const prisma = new PrismaClient({ adapter });
export default prisma;
