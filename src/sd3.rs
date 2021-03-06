// 线段处理标准模式
// 选择一个起点后，首先要做的是把起始线段处理成标准线段模式，处理的方向，是要按起点前面一条线段的特征序列来。
// 比如，找到一个起点后，从起点开始的线段是向上笔开始的，不管前面有没有别的线段，就想像前面一定是个下跌线段，下跌线段的特征序列是向上笔，那么这个起点开始后的线段，也是先按向上笔来包含处理
// 特征序列包含处理规则：一是，不管是前包含还是后包含，**一律从前特征序列的起点到后特征序列的终点**；二是，在新的标准线段模式出现前，只做向后处理，不做向前处理
// 在进行线段划分的时候，先不要管是不是有缺口，先按特征序列向后包含处理，一直到出现新的标准线段再说，这个时候，其实就会出现缠师说的分型，分型不出现，新的标准线段是不会出现的，
// 这个时候，我们再来看，处理过后的标准特征序列是不是有缺口，再来考察是不是要按缠师说的两种情况进行调整。如此，这个线段划分程序才是最清晰的，否则就是乱的。
// 也就是有特征序列缺口的情况，如何找反向分型的。前面的步骤是不变的，先向后处理，找出新的标准线段再说，这时候，再看第一和第二标准特征序列之间是否有缺口，
// 如果有缺口，那前面那个线段还不能确认结束，先别管它，按这个新的标准线段的特征序列开始找是否形成分型，这里面的包含关系就是不管是前包含，还是后包含，都要处理，处理规则还是一样的，
// 从前面特征序列的起点到后特征序列的终点，处理完以后，还没有分型，就出了新高或新低，就判前面那个已经死了的线段又活过来了，出现了分型，前面那个线段就判定它彻底死。

// 线段的划分，其实是分两步，两种情况的，第一步是要找出新的标准线段，第二步就是分两种情况来判断前线段是否结束，如果判定前线段结束了，那么就不管它了，就以新的标准线段为基准，来找下一个新的标准线段的出现，
// 特别注意这其中的特征序列的应用是有个细微的差别的，区别就在于起始点开始的线段特征序列是按前面假设的线段来的，后面这种一但判断前线段结束后，前面的特征序列就不用了，而是按新线段的特征序列来的。至于说缺口的情况，只是第二步骤中的一种情况.

// 再次总结:
// 先变成标准线段
//
