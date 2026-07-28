-- CreateEnum
CREATE TYPE "VotingOption" AS ENUM ('DEFINITLY_NOT', 'MAYBE_NEED_TRYING', 'NOT_ABLE_TO_PLAY', 'DOES_NOT_SUIT', 'NOT_WHOLE_SONG');

-- CreateTable
CREATE TABLE "WishlistSong" (
    "id" TEXT NOT NULL,
    "title" TEXT NOT NULL,
    "artist" TEXT NOT NULL,
    "comment" TEXT,
    "link" TEXT,

    CONSTRAINT "WishlistSong_pkey" PRIMARY KEY ("id")
);

-- CreateTable
CREATE TABLE "WishlistSongVoting" (
    "wishlistSongId" TEXT NOT NULL,
    "userId" TEXT NOT NULL,
    "option" "VotingOption",
    "comment" TEXT,

    CONSTRAINT "WishlistSongVoting_pkey" PRIMARY KEY ("wishlistSongId","userId")
);

-- AddForeignKey
ALTER TABLE "WishlistSongVoting" ADD CONSTRAINT "WishlistSongVoting_wishlistSongId_fkey" FOREIGN KEY ("wishlistSongId") REFERENCES "WishlistSong"("id") ON DELETE RESTRICT ON UPDATE CASCADE;

-- AddForeignKey
ALTER TABLE "WishlistSongVoting" ADD CONSTRAINT "WishlistSongVoting_userId_fkey" FOREIGN KEY ("userId") REFERENCES "User"("id") ON DELETE RESTRICT ON UPDATE CASCADE;
