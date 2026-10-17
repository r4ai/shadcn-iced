import type { Fixtures } from "@/fixtures"
import { Card, CardContent } from "@/components/ui/card"
import {
  Carousel,
  CarouselContent,
  CarouselItem,
  CarouselNext,
  CarouselPrevious,
} from "@/components/ui/carousel"

const slides = Array.from({ length: 5 }, (_, index) => index + 1)

const demo = () => (
  <div className="px-12">
    <Carousel className="w-full max-w-xs">
      <CarouselContent>
        {slides.map((n) => (
          <CarouselItem key={n}>
            <div className="p-1">
              <Card>
                <CardContent className="flex aspect-square items-center justify-center p-6">
                  <span className="text-4xl font-semibold">{n}</span>
                </CardContent>
              </Card>
            </div>
          </CarouselItem>
        ))}
      </CarouselContent>
      <CarouselPrevious />
      <CarouselNext />
    </Carousel>
  </div>
)

export default {
  default: demo,
  keyboard: demo,
  sizes: () => (
    <div className="px-12">
      <Carousel opts={{ align: "start" }} className="w-full max-w-sm">
        <CarouselContent>
          {slides.map((n) => (
            <CarouselItem key={n} className="basis-1/3">
              <div className="p-1">
                <Card>
                  <CardContent className="flex aspect-square items-center justify-center p-6">
                    <span className="text-3xl font-semibold">{n}</span>
                  </CardContent>
                </Card>
              </div>
            </CarouselItem>
          ))}
        </CarouselContent>
        <CarouselPrevious />
        <CarouselNext />
      </Carousel>
    </div>
  ),
  vertical: () => (
    <div className="py-12">
      <Carousel
        opts={{ align: "start" }}
        orientation="vertical"
        className="w-full max-w-xs"
      >
        <CarouselContent className="-mt-1 h-[200px]">
          {slides.map((n) => (
            <CarouselItem key={n} className="basis-1/2 pt-1">
              <div className="p-1">
                <Card>
                  <CardContent className="flex items-center justify-center p-6">
                    <span className="text-3xl font-semibold">{n}</span>
                  </CardContent>
                </Card>
              </div>
            </CarouselItem>
          ))}
        </CarouselContent>
        <CarouselPrevious />
        <CarouselNext />
      </Carousel>
    </div>
  ),
} satisfies Fixtures
